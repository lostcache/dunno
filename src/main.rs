use clap::Parser;

/// Application entry point.
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

/// Main application logic dispatcher.
async fn run(args: dunno::args::Args) -> anyhow::Result<()> {
    let config = dunno::config::Config::load(args.backend.as_deref())?;

    if let dunno::args::Commands::Config { command } = &args.command {
        return handle_config_command(command, &config);
    }

    let db = dunno::db::DB::from_config(&config).await?;
    dispatch_command(args.command, &db).await
}

/// Routes commands to their specialized handlers.
async fn dispatch_command(
    command: dunno::args::Commands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
        dunno::args::Commands::Add {
            field_names,
            field_values,
            link_to,
        } => handle_add(field_names, field_values, link_to, db).await,
        dunno::args::Commands::Link {
            from_id,
            edge,
            to_ids,
        } => handle_link(from_id, edge, to_ids, db).await,
        dunno::args::Commands::Project { command } => handle_project_command(command, db).await,
        dunno::args::Commands::Module { command } => handle_module_command(command, db).await,
        dunno::args::Commands::Submodule { command } => handle_submodule_command(command, db).await,
        dunno::args::Commands::File { command } => handle_file_command(command, db).await,
        dunno::args::Commands::Task { command } => handle_task_command(command, db).await,
        dunno::args::Commands::Subtask { command } => handle_subtask_command(command, db).await,
        dunno::args::Commands::Todo { command } => handle_todo_command(command, db).await,
        dunno::args::Commands::UserStory { command } => {
            handle_user_story_command(command, db).await
        }
        dunno::args::Commands::Epic { command } => handle_epic_command(command, db).await,
        dunno::args::Commands::Context {
            task_id,
            file_id,
            subtask_id,
            epic_id,
        } => handle_context(task_id, file_id, subtask_id, epic_id, db).await,
        dunno::args::Commands::Purge => handle_purge(db).await,
        dunno::args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}

/// Handles config display without requiring database connection.
fn handle_config_command(
    command: &dunno::args::ConfigCommands,
    config: &dunno::config::Config,
) -> anyhow::Result<()> {
    match command {
        dunno::args::ConfigCommands::Show => {
            println!("{}", config.redacted_json());
        }
    }
    Ok(())
}

/// Ingests new knowledge items into the system.
async fn handle_add(
    field_names: Vec<String>,
    field_values: Vec<String>,
    link_to: Vec<String>,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    // Validate that field_names and field_values have the same length
    if field_names.len() != field_values.len() {
        return Err(anyhow::anyhow!(
            "Number of --field flags ({}) must match number of --value flags ({})",
            field_names.len(),
            field_values.len()
        ));
    }

    // Build a JSON object from paired --field and --value flags
    let mut map = serde_json::Map::new();
    for (key, value) in field_names.into_iter().zip(field_values.into_iter()) {
        map.insert(key, serde_json::Value::String(value));
    }
    dunno::ingest::add_knowledge_schemaless(map, link_to, db).await?;
    print_success();
    Ok(())
}

/// Validates edge types against allowed schema before creating links.
async fn handle_link(
    from_id: String,
    edge: String,
    to_ids: Vec<String>,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
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

    print_success();
    Ok(())
}

/// Project management commands.
async fn handle_project_command(
    command: dunno::args::ProjectCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
    }
    Ok(())
}

/// Module management with multi-project linking support.
async fn handle_module_command(
    command: dunno::args::ModuleCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
    }
    Ok(())
}

/// Submodule management with optional module filtering.
async fn handle_submodule_command(
    command: dunno::args::SubmoduleCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
            let submodules = match module_id {
                Some(mid) => db.list_submodules_by_module(&mid).await?,
                None => db.list_submodules().await?,
            };
            println!("{}", serde_json::json!(submodules));
        }
    }
    Ok(())
}

/// File management with parent hierarchy linking.
async fn handle_file_command(
    command: dunno::args::FileCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
            let files = match (module_id, submodule_id) {
                (Some(mid), _) => db.list_files_by_module(&mid).await?,
                (_, Some(sid)) => db.list_files_by_submodule(&sid).await?,
                (None, None) => db.list_files().await?,
            };
            println!("{}", serde_json::json!(files));
        }
    }
    Ok(())
}

/// Task lifecycle management with relationship linking.
async fn handle_task_command(
    command: dunno::args::TaskCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
        dunno::args::TaskCommands::Create {
            module_ids,
            project_ids,
            user_story_ids,
            epic_ids,
            name,
            description,
        } => {
            let (mid, pid) = validate_task_parents(&module_ids, &project_ids)?;
            let created = db.create_task(&name, &description, mid, pid).await?;

            if let Some(task_id) = &created.id {
                for us_id in &user_story_ids {
                    db.link_task_to_user_story(task_id, us_id).await?;
                }
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
            let parsed_status = parse_optional_status(status)?;
            let updated = db
                .update_task(&task_id, name, description, parsed_status)
                .await?;

            match updated {
                Some(task) => println!("{}", serde_json::json!(task)),
                None => return Err(anyhow::anyhow!("Task not found: {}", task_id)),
            }
        }
        dunno::args::TaskCommands::List => {
            let tasks = db.list_tasks().await?;
            println!("{}", serde_json::json!(tasks));
        }
        dunno::args::TaskCommands::Delete { task_id } => {
            let deleted = db.delete_task(&task_id).await?;
            if deleted {
                println!("{}", serde_json::json!({ "status": "ok", "deleted": task_id }));
            } else {
                return Err(anyhow::anyhow!("Task not found: {}", task_id));
            }
        }
    }
    Ok(())
}

/// Validates task parent constraints: either freestanding (no parents) or
/// exactly one module and one project.
fn validate_task_parents<'a>(
    module_ids: &'a [String],
    project_ids: &'a [String],
) -> anyhow::Result<(Option<&'a str>, Option<&'a str>)> {
    match (module_ids.len(), project_ids.len()) {
        (0, 0) => Ok((None, None)),
        (1, 1) => Ok((Some(&module_ids[0]), Some(&project_ids[0]))),
        _ => Err(anyhow::anyhow!(
            "Task create: provide either no module/project IDs (freestanding) or exactly one of each (linked). Got {} module_ids and {} project_ids",
            module_ids.len(),
            project_ids.len()
        )),
    }
}

/// Parses optional status string into typed TaskStatus.
fn parse_optional_status(
    status: Option<String>,
) -> anyhow::Result<Option<dunno::models::TaskStatus>> {
    match status {
        Some(value) => dunno::models::TaskStatus::parse(&value)
            .map(Some)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid status '{}'. Expected: not_started, started, finished",
                    value
                )
            }),
        None => Ok(None),
    }
}

/// Subtask management with bidirectional task linking.
async fn handle_subtask_command(
    command: dunno::args::SubtaskCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
    }
    Ok(())
}

/// Todo management with project association.
async fn handle_todo_command(
    command: dunno::args::TodoCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
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
    }
    Ok(())
}

/// User story management with epic linking.
async fn handle_user_story_command(
    command: dunno::args::UserStoryCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
        dunno::args::UserStoryCommands::Create {
            project_id,
            epic_ids,
            title,
            description,
        } => {
            let created = db
                .create_user_story(&title, &description, &project_id)
                .await?;

            if let Some(us_id) = &created.id {
                for epic_id in &epic_ids {
                    db.link_user_story_to_epic(us_id, epic_id).await?;
                }
            }

            println!("{}", serde_json::json!(created));
        }
        dunno::args::UserStoryCommands::List {
            project_id,
            epic_id,
        } => {
            let user_stories = match (epic_id, project_id) {
                (Some(eid), _) => db.list_user_stories_by_epic(&eid).await?,
                (_, Some(pid)) => db.list_user_stories_by_project(&pid).await?,
                (None, None) => db.list_user_stories().await?,
            };
            println!("{}", serde_json::json!(user_stories));
        }
    }
    Ok(())
}

/// Epic management for project-level feature grouping.
async fn handle_epic_command(
    command: dunno::args::EpicCommands,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    match command {
        dunno::args::EpicCommands::Create {
            project_id,
            title,
            description,
        } => {
            let created = db.create_epic(&title, &description, &project_id).await?;
            println!("{}", serde_json::json!(created));
        }
        dunno::args::EpicCommands::List { project_id } => {
            let epics = match project_id {
                Some(pid) => db.list_epics_by_project(&pid).await?,
                None => db.list_epics().await?,
            };
            println!("{}", serde_json::json!(epics));
        }
    }
    Ok(())
}

/// Context gathering for various entity types.
async fn handle_context(
    task_id: Option<String>,
    file_id: Option<String>,
    subtask_id: Option<String>,
    epic_id: Option<String>,
    db: &dunno::db::DB,
) -> anyhow::Result<()> {
    let results = match (task_id, file_id, subtask_id, epic_id) {
        (Some(t_id), _, _, _) => dunno::context::get_task_context(&t_id, db).await?,
        (_, Some(f_id), _, _) => dunno::context::get_file_context(&f_id, db).await?,
        (_, _, Some(st_id), _) => dunno::context::get_subtask_context(&st_id, db).await?,
        (_, _, _, Some(e_id)) => dunno::context::get_epic_context(&e_id, db).await?,
        (None, None, None, None) => {
            return Err(anyhow::anyhow!(
                "One of --task-id, --file-id, --subtask-id, or --epic-id must be provided"
            ));
        }
    };

    println!("{}", serde_json::json!({ "results": results }));
    Ok(())
}

/// Destructive operation to clear all data.
async fn handle_purge(db: &dunno::db::DB) -> anyhow::Result<()> {
    db.purge_database().await?;
    println!(
        "{}",
        serde_json::json!({
            "status": "ok",
            "message": "Database purged successfully"
        })
    );
    Ok(())
}

/// Prints standardized success response.
fn print_success() {
    println!("{}", serde_json::json!({ "status": "ok" }));
}

/// Prints machine-readable JSON error for CLI integration.
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
