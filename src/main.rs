use clap::Parser;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let args = match dunno::args::Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                print!("{}", err);
                return;
            }
            print_error_json("cli_parse_error", err.to_string());
            std::process::exit(2);
        }
    };

    if let Err(err) = run(args).await {
        print_error_json("runtime_error", err.to_string());
        std::process::exit(1);
    }
}

async fn run(args: dunno::args::Args) -> anyhow::Result<()> {
    let config = dunno::config::Config::load(args.backend.as_deref())?;
    if let dunno::args::Commands::Config { command } = &args.command {
        match command {
            dunno::args::ConfigCommands::Show => {
                println!("{}", config.redacted_json());
            }
        }
        return Ok(());
    }

    let db = dunno::db::DB::from_config(&config).await?;

    match args.command {
        dunno::args::Commands::Add {
            kind,
            content,
            link_to,
        } => {
            dunno::ingest::add_knowledge(kind, content, link_to, &db).await?;
            println!("{}", serde_json::json!({ "status": "ok" }));
        }
        dunno::args::Commands::Link {
            from_id,
            edge,
            to_ids,
        } => {
            const ALLOWED_EDGES: &[&str] = &[
                "contains",
                "has_task",
                "has_subtask",
                "has_todo",
                "has_context",
                "has_user_story",
                "has_module",
                "has_submodule",
                "has_epic",
                "belongs_to_project",
                "belongs_to_module",
                "belongs_to_task",
                "belongs_to_story",
                "belongs_to_user_story",
                "belongs_to_epic",
            ];
            if !ALLOWED_EDGES.contains(&edge.as_str()) {
                return Err(anyhow::anyhow!(
                    "Unknown edge {:?}. Allowed: {:?}",
                    edge,
                    ALLOWED_EDGES
                ));
            }
            if to_ids.is_empty() {
                return Err(anyhow::anyhow!("At least one --to ID is required"));
            }
            for to_id in &to_ids {
                db.link(&from_id, &edge, to_id).await?;
            }
            println!("{}", serde_json::json!({ "status": "ok" }));
        }
        dunno::args::Commands::Project { command } => match command {
            dunno::args::ProjectCommands::Create { name, description } => {
                let project = dunno::models::Project {
                    id: None,
                    name,
                    description,
                };
                let created = db.create_project(&project).await?;
                println!("{}", serde_json::json!(created));
            }
            dunno::args::ProjectCommands::List => {
                let projects = db.list_projects().await?;
                println!("{}", serde_json::json!(projects));
            }
        },
        dunno::args::Commands::Module { command } => match command {
            dunno::args::ModuleCommands::Create {
                project_ids,
                name,
                description,
            } => {
                let created = db
                    .create_module(&name, &description, project_ids.first().map(String::as_str))
                    .await?;
                let module_id = match &created.id {
                    Some(id) => id.as_str(),
                    None => {
                        println!("{}", serde_json::json!(created));
                        return Ok(());
                    }
                };
                for pid in project_ids.iter().skip(1) {
                    db.link(pid, "contains", module_id).await?;
                }
                println!("{}", serde_json::json!(created));
            }
            dunno::args::ModuleCommands::List => {
                let modules = db.list_modules().await?;
                println!("{}", serde_json::json!(modules));
            }
        },
        dunno::args::Commands::Submodule { command } => match command {
            dunno::args::SubmoduleCommands::Create {
                module_ids,
                name,
                description,
            } => {
                let created = db
                    .create_submodule(&name, &description, module_ids.first().map(String::as_str))
                    .await?;
                let sub_id = match &created.id {
                    Some(id) => id.as_str(),
                    None => {
                        println!("{}", serde_json::json!(created));
                        return Ok(());
                    }
                };
                for mid in module_ids.iter().skip(1) {
                    db.link(mid, "contains", sub_id).await?;
                }
                println!("{}", serde_json::json!(created));
            }
            dunno::args::SubmoduleCommands::List { module_id } => {
                let submodules = if let Some(mid) = module_id {
                    db.list_submodules_by_module(&mid).await?
                } else {
                    db.list_submodules().await?
                };
                println!("{}", serde_json::json!(submodules));
            }
        },
        dunno::args::Commands::File { command } => match command {
            dunno::args::FileCommands::Create {
                parent_ids,
                name,
                path,
            } => {
                let created = db
                    .create_file(&name, &path, parent_ids.first().map(String::as_str))
                    .await?;
                let file_id = match &created.id {
                    Some(id) => id.as_str(),
                    None => {
                        println!("{}", serde_json::json!(created));
                        return Ok(());
                    }
                };
                for pid in parent_ids.iter().skip(1) {
                    db.link(pid, "contains", file_id).await?;
                }
                println!("{}", serde_json::json!(created));
            }
            dunno::args::FileCommands::List {
                module_id,
                submodule_id,
            } => {
                let files = if let Some(mid) = module_id {
                    db.list_files_by_module(&mid).await?
                } else if let Some(sid) = submodule_id {
                    db.list_files_by_submodule(&sid).await?
                } else {
                    db.list_files().await?
                };
                println!("{}", serde_json::json!(files));
            }
        },
        dunno::args::Commands::Task { command } => match command {
            dunno::args::TaskCommands::Create {
                module_ids,
                project_ids,
                user_story_ids,
                epic_ids,
                name,
                description,
            } => {
                let (mid, pid) = match (module_ids.len(), project_ids.len()) {
                    (0, 0) => (None, None),
                    (1, 1) => (Some(module_ids[0].as_str()), Some(project_ids[0].as_str())),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Task create: provide either no module/project IDs (freestanding) or exactly one of each (linked). Got {} module_ids and {} project_ids",
                            module_ids.len(),
                            project_ids.len()
                        ));
                    }
                };
                let created = db.create_task(&name, &description, mid, pid).await?;
                
                // Link task to user stories if provided
                if let Some(task_id) = &created.id {
                    for us_id in &user_story_ids {
                        db.link_task_to_user_story(task_id, us_id).await?;
                    }
                    // Link task to epics if provided
                    for epic_id in &epic_ids {
                        db.link_task_to_epic(task_id, epic_id).await?;
                    }
                }
                
                println!("{}", serde_json::json!(created));
            }
            dunno::args::TaskCommands::Update {
                task_id,
                name,
                description,
                status,
            } => {
                let parsed_status = match status {
                    Some(value) => {
                        Some(dunno::models::TaskStatus::parse(&value).ok_or_else(|| {
                            anyhow::anyhow!(
                                "Invalid status '{}'. Expected: not_started, started, finished",
                                value
                            )
                        })?)
                    }
                    None => None,
                };
                let updated = db
                    .update_task(&task_id, name, description, parsed_status)
                    .await?;
                if let Some(task) = updated {
                    println!("{}", serde_json::json!(task));
                } else {
                    return Err(anyhow::anyhow!("Task not found: {}", task_id));
                }
            }
            dunno::args::TaskCommands::List => {
                let tasks = db.list_tasks().await?;
                println!("{}", serde_json::json!(tasks));
            }
        },
        dunno::args::Commands::Subtask { command } => match command {
            dunno::args::SubtaskCommands::Create {
                task_ids,
                name,
                description,
            } => {
                let created = db
                    .create_subtask(&name, &description, task_ids.first().map(String::as_str))
                    .await?;
                let stid = match &created.id {
                    Some(id) => id.as_str(),
                    None => {
                        println!("{}", serde_json::json!(created));
                        return Ok(());
                    }
                };
                for tid in task_ids.iter().skip(1) {
                    db.link(tid, "has_subtask", stid).await?;
                    db.link(stid, "belongs_to_task", tid).await?;
                }
                println!("{}", serde_json::json!(created));
            }
            dunno::args::SubtaskCommands::List { task_id } => {
                let subtasks = db.list_subtasks_by_task(&task_id).await?;
                println!("{}", serde_json::json!(subtasks));
            }
        },
        dunno::args::Commands::Todo { command } => match command {
            dunno::args::TodoCommands::Create {
                project_ids,
                content,
            } => {
                let created = db
                    .create_todo(&content, project_ids.first().map(String::as_str))
                    .await?;
                let todo_id = match &created.id {
                    Some(id) => id.as_str(),
                    None => {
                        println!("{}", serde_json::json!(created));
                        return Ok(());
                    }
                };
                for pid in project_ids.iter().skip(1) {
                    db.link(pid, "has_todo", todo_id).await?;
                }
                println!("{}", serde_json::json!(created));
            }
            dunno::args::TodoCommands::List { project_id } => {
                let todos = db.list_todos_by_project(&project_id).await?;
                println!("{}", serde_json::json!(todos));
            }
        },
        dunno::args::Commands::UserStory { command } => match command {
            dunno::args::UserStoryCommands::Create {
                project_id,
                epic_ids,
                title,
                description,
            } => {
                let created = db.create_user_story(&title, &description, &project_id).await?;
                
                // Link user story to epics if provided
                if let Some(us_id) = &created.id {
                    for epic_id in &epic_ids {
                        db.link_user_story_to_epic(us_id, epic_id).await?;
                    }
                }
                
                println!("{}", serde_json::json!(created));
            }
            dunno::args::UserStoryCommands::List { project_id, epic_id } => {
                let user_stories = if let Some(eid) = epic_id {
                    db.list_user_stories_by_epic(&eid).await?
                } else if let Some(pid) = project_id {
                    db.list_user_stories_by_project(&pid).await?
                } else {
                    db.list_user_stories().await?
                };
                println!("{}", serde_json::json!(user_stories));
            }
        },
        dunno::args::Commands::Epic { command } => match command {
            dunno::args::EpicCommands::Create {
                project_id,
                title,
                description,
            } => {
                let created = db.create_epic(&title, &description, &project_id).await?;
                println!("{}", serde_json::json!(created));
            }
            dunno::args::EpicCommands::List { project_id } => {
                let epics = if let Some(pid) = project_id {
                    db.list_epics_by_project(&pid).await?
                } else {
                    db.list_epics().await?
                };
                println!("{}", serde_json::json!(epics));
            }
        },
        dunno::args::Commands::Context {
            task_id,
            file_id,
            subtask_id,
            epic_id,
        } => {
            if let Some(t_id) = task_id {
                let results = dunno::context::get_task_context(&t_id, &db).await?;
                println!("{}", serde_json::json!({ "results": results }));
            } else if let Some(f_id) = file_id {
                let results = dunno::context::get_file_context(&f_id, &db).await?;
                println!("{}", serde_json::json!({ "results": results }));
            } else if let Some(st_id) = subtask_id {
                let results = dunno::context::get_subtask_context(&st_id, &db).await?;
                println!("{}", serde_json::json!({ "results": results }));
            } else if let Some(e_id) = epic_id {
                let results = dunno::context::get_epic_context(&e_id, &db).await?;
                println!("{}", serde_json::json!({ "results": results }));
            } else {
                return Err(anyhow::anyhow!(
                    "One of --task-id, --file-id, --subtask-id, or --epic-id must be provided"
                ));
            }
        }
        dunno::args::Commands::Purge => {
            db.purge_database().await?;
            println!(
                "{}",
                serde_json::json!({ "status": "ok", "message": "Database purged successfully" })
            );
        }
        dunno::args::Commands::Config { .. } => {
            unreachable!("config command returns before DB init")
        }
    }

    Ok(())
}

fn print_error_json(kind: &str, message: String) {
    println!(
        "{}",
        serde_json::json!({
            "status": "error",
            "kind": kind,
            "error": message
        })
    );
}
