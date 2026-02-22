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
                project_id,
                name,
                description,
            } => {
                let created = db.create_module(&name, &description, &project_id).await?;
                println!("{}", serde_json::json!(created));
            }
            dunno::args::ModuleCommands::List => {
                let modules = db.list_modules().await?;
                println!("{}", serde_json::json!(modules));
            }
        },
        dunno::args::Commands::Submodule { command } => match command {
            dunno::args::SubmoduleCommands::Create {
                module_id,
                name,
                description,
            } => {
                let created = db.create_submodule(&name, &description, &module_id).await?;
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
                parent_id,
                name,
                path,
            } => {
                let created = db.create_file(&name, &path, &parent_id).await?;
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
                module_id,
                project_id,
                name,
                description,
            } => {
                let created = db.create_task(&name, &description, &module_id, &project_id).await?;
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
                task_id,
                name,
                description,
            } => {
                let created = db.create_subtask(&name, &description, &task_id).await?;
                println!("{}", serde_json::json!(created));
            }
            dunno::args::SubtaskCommands::List { task_id } => {
                let subtasks = db.list_subtasks_by_task(&task_id).await?;
                println!("{}", serde_json::json!(subtasks));
            }
        },
        dunno::args::Commands::Todo { command } => match command {
            dunno::args::TodoCommands::Create {
                project_id,
                content,
            } => {
                let created = db.create_todo(&content, &project_id).await?;
                println!("{}", serde_json::json!(created));
            }
            dunno::args::TodoCommands::List { project_id } => {
                let todos = db.list_todos_by_project(&project_id).await?;
                println!("{}", serde_json::json!(todos));
            }
        },
        dunno::args::Commands::Context {
            task_id,
            file_id,
            subtask_id,
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
            } else {
                return Err(anyhow::anyhow!(
                    "One of --task-id, --file-id, or --subtask-id must be provided"
                ));
            }
        }
        dunno::args::Commands::Purge => {
            db.purge_database().await?;
            println!("{}", serde_json::json!({ "status": "ok", "message": "Database purged successfully" }));
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
