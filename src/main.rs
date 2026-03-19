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
        return handle_config_command(command, &config, args.pretty);
    }

    let db = dunno::db::DB::from_config(&config).await?;
    dispatch_command(args.command, &db, args.pretty, args.ignore_case).await
}

/// Routes commands to their specialized handlers.
async fn dispatch_command(
    command: dunno::args::Commands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::Commands::Add {
            field_names,
            field_values,
            link_to,
        } => handle_add(field_names, field_values, link_to, db, pretty).await,
        dunno::args::Commands::Link {
            from_id,
            edge,
            to_ids,
        } => handle_link(from_id, edge, to_ids, db, pretty).await,
        dunno::args::Commands::Project { command } => {
            handle_project_command(command, db, pretty).await
        }
        dunno::args::Commands::Module { command } => {
            handle_module_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Submodule { command } => {
            handle_submodule_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::File { command } => {
            handle_file_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Task { command } => {
            handle_task_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Todo { command } => {
            handle_todo_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::UserStory { command } => {
            handle_user_story_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Epic { command } => {
            handle_epic_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Persona { command } => {
            handle_persona_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Workflow { command } => {
            handle_workflow_command(command, db, pretty, ignore_case).await
        }
        dunno::args::Commands::Context {
            task_id,
            file_id,
            epic_id,
            full,
        } => handle_context(task_id, file_id, epic_id, full, db, pretty).await,
        dunno::args::Commands::Purge => handle_purge(db, pretty).await,
        dunno::args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}

/// Handles config display without requiring database connection.
fn handle_config_command(
    command: &dunno::args::ConfigCommands,
    config: &dunno::config::Config,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::ConfigCommands::Show => {
            if pretty {
                print!("{}", config.formatted());
            } else {
                println!("{}", config.redacted_json());
            }
        }
    }
    Ok(())
}

/// Resolves a project identifier (ID or name) to a project ID.
/// If `project_id` is provided, returns it directly.
/// If `project_name` is provided, looks up the project by name.
async fn resolve_project_id(
    db: &dunno::db::DB,
    project_id: Option<String>,
    project_name: Option<String>,
    ignore_case: bool,
) -> anyhow::Result<Option<String>> {
    match (project_id, project_name) {
        (Some(id), _) => Ok(Some(id)),
        (None, Some(name)) => {
            let project = db
                .get_project_by_name(&name, ignore_case)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to lookup project by name: {}", e))?;
            match project {
                Some(p) => Ok(p.id),
                None => Err(anyhow::anyhow!("Project not found: {}", name)),
            }
        }
        (None, None) => Ok(None),
    }
}
async fn handle_add(
    field_names: Vec<String>,
    field_values: Vec<String>,
    link_to: Vec<String>,
    db: &dunno::db::DB,
    pretty: bool,
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
    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}

/// Validates edge types against allowed schema before creating links.
async fn handle_link(
    from_id: String,
    edge: String,
    to_ids: Vec<String>,
    db: &dunno::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    const ALLOWED_EDGES: &[&str] = &[
        "contains",
        "has_task",
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

    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}

/// Project management commands.
async fn handle_project_command(
    command: dunno::args::ProjectCommands,
    db: &dunno::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::ProjectCommands::Create { name, description } => {
            let project = dunno::models::Project {
                id: None,
                name,
                description,
            };
            let created = db.create_project(&project).await?;
            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::ProjectCommands::List => {
            let projects = db.list_projects().await?;
            print_json(serde_json::json!(projects), pretty);
        }
        dunno::args::ProjectCommands::Delete { project_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in project_ids {
                if db.delete_project(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Module management with multi-project linking support.
async fn handle_module_command(
    command: dunno::args::ModuleCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::ModuleCommands::Create {
            project_ids,
            project,
            name,
            description,
            notes,
        } => {
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;
            let project_id = resolved_project_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-ids/--pids or --project/-p must be provided")
            })?;

            let created = db
                .create_module(
                    &name,
                    &description,
                    notes.as_deref(),
                    &project_id,
                )
                .await?;
            let module_id = match &created.id {
                Some(id) => id.as_str(),
                None => {
                    print_json(serde_json::json!(created), pretty);
                    return Ok(());
                }
            };

            // Link additional project IDs (from project_ids, skipping the first which was already handled)
            for pid in project_ids.iter().skip(1) {
                db.link(pid, "contains", module_id).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::ModuleCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let modules = match resolved_id {
                Some(pid) => db.list_modules_by_project(&pid).await?,
                None => db.list_modules().await?,
            };
            print_json(serde_json::json!(modules), pretty);
        }
        dunno::args::ModuleCommands::Delete { module_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in module_ids {
                if db.delete_module(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Submodule management with optional module and project filtering.
async fn handle_submodule_command(
    command: dunno::args::SubmoduleCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::SubmoduleCommands::Create {
            project_ids,
            project,
            module_ids,
            name,
            description,
            notes,
        } => {
            if module_ids.is_empty() {
                anyhow::bail!("--module-ids/--mids must be provided");
            }
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;
            let project_id = resolved_project_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-ids/--pids or --project/-p must be provided")
            })?;

            let created = db
                .create_submodule(
                    &name,
                    &description,
                    notes.as_deref(),
                    module_ids.first().map(String::as_str).unwrap(),
                    &project_id,
                )
                .await?;

            let sub_id = match &created.id {
                Some(id) => id.as_str(),
                None => {
                    print_json(serde_json::json!(created), pretty);
                    return Ok(());
                }
            };

            for mid in module_ids.iter().skip(1) {
                db.link(mid, "contains", sub_id).await?;
            }
            for pid in project_ids.iter().skip(1) {
                db.link(sub_id, "belongs_to_project", pid).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::SubmoduleCommands::List {
            project_id,
            project,
            module_id,
        } => {
            let submodules = match module_id {
                Some(mid) => db.list_submodules_by_module(&mid).await?,
                None => {
                    let resolved_id =
                        resolve_project_id(db, project_id, project, ignore_case).await?;
                    match resolved_id {
                        Some(pid) => db.list_submodules_by_project(&pid).await?,
                        None => db.list_submodules().await?,
                    }
                }
            };
            print_json(serde_json::json!(submodules), pretty);
        }
        dunno::args::SubmoduleCommands::Delete { submodule_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in submodule_ids {
                if db.delete_submodule(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// File management with parent hierarchy linking and project filtering.
async fn handle_file_command(
    command: dunno::args::FileCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::FileCommands::Create {
            project_ids,
            project,
            parent_ids,
            name,
            path,
            description,
            notes,
        } => {
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;
            let project_id = resolved_project_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-ids/--pids or --project/-p must be provided")
            })?;
            let created = db
                .create_file(
                    &name,
                    &path,
                    description.as_deref(),
                    notes.as_deref(),
                    &project_id,
                    parent_ids.first().map(String::as_str),
                )
                .await?;

            let file_id = match &created.id {
                Some(id) => id.as_str(),
                None => {
                    print_json(serde_json::json!(created), pretty);
                    return Ok(());
                }
            };

            for pid in parent_ids.iter().skip(1) {
                db.link(pid, "contains", file_id).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::FileCommands::List {
            project_id,
            project,
            module_id,
            submodule_id,
        } => {
            let files = match (submodule_id, module_id, project_id, project) {
                (Some(sid), _, _, _) => db.list_files_by_submodule(&sid).await?,
                (_, Some(mid), _, _) => db.list_files_by_module(&mid).await?,
                (_, _, pid_opt, proj_opt) => {
                    let resolved_id =
                        resolve_project_id(db, pid_opt, proj_opt, ignore_case).await?;
                    match resolved_id {
                        Some(pid) => db.list_files_by_project(&pid).await?,
                        None => db.list_files().await?,
                    }
                }
            };
            print_json(serde_json::json!(files), pretty);
        }
        dunno::args::FileCommands::Delete { file_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in file_ids {
                if db.delete_file(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Task lifecycle management with relationship linking.
async fn handle_task_command(
    command: dunno::args::TaskCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::TaskCommands::Create {
            module_ids,
            project_ids,
            project,
            user_story_ids,
            epic_ids,
            name,
            description,
        } => {
            // Resolve project name to ID if provided
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;

            // Convert resolved ID back to Vec<String> for compatibility
            let effective_project_ids: Vec<String> = match resolved_project_id {
                Some(id) => vec![id],
                None => project_ids,
            };

            let (mid, pid) = validate_task_parents(&module_ids, &effective_project_ids)?;
            let created = db.create_task(&name, &description, mid, pid).await?;

            if let Some(task_id) = &created.id {
                for us_id in &user_story_ids {
                    db.link_task_to_user_story(task_id, us_id).await?;
                }
                for epic_id in &epic_ids {
                    db.link_task_to_epic(task_id, epic_id).await?;
                }
            }

            print_json(serde_json::json!(created), pretty);
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
                Some(task) => print_json(serde_json::json!(task), pretty),
                None => return Err(anyhow::anyhow!("Task not found: {}", task_id)),
            }
        }
        dunno::args::TaskCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let tasks = match resolved_id {
                Some(pid) => db.list_tasks_by_project(&pid).await?,
                None => db.list_tasks().await?,
            };
            print_json(serde_json::json!(tasks), pretty);
        }
        dunno::args::TaskCommands::Delete { task_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in task_ids {
                if db.delete_task(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
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

/// Todo management with project association.
async fn handle_todo_command(
    command: dunno::args::TodoCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::TodoCommands::Create {
            project_ids,
            project,
            content,
        } => {
            // Resolve project name to ID if provided
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;

            let created = db
                .create_todo(&content, resolved_project_id.as_deref())
                .await?;

            let todo_id = match &created.id {
                Some(id) => id.as_str(),
                None => {
                    print_json(serde_json::json!(created), pretty);
                    return Ok(());
                }
            };

            // Link additional project IDs (from project_ids, skipping the first which was already handled)
            for pid in project_ids.iter().skip(1) {
                db.link(pid, "has_todo", todo_id).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::TodoCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;
            let todos = db.list_todos_by_project(&pid).await?;
            print_json(serde_json::json!(todos), pretty);
        }
        dunno::args::TodoCommands::Delete { todo_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in todo_ids {
                if db.delete_todo(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// User story management with epic linking.
async fn handle_user_story_command(
    command: dunno::args::UserStoryCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::UserStoryCommands::Create {
            project_id,
            project,
            epic_ids,
            title,
            description,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;

            let created = db.create_user_story(&title, &description, &pid).await?;

            if let Some(us_id) = &created.id {
                for epic_id in &epic_ids {
                    db.link_user_story_to_epic(us_id, epic_id).await?;
                }
            }

            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::UserStoryCommands::List {
            project_id,
            project,
            epic_id,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let user_stories = match (epic_id, resolved_id) {
                (Some(eid), _) => db.list_user_stories_by_epic(&eid).await?,
                (_, Some(pid)) => db.list_user_stories_by_project(&pid).await?,
                (None, None) => db.list_user_stories().await?,
            };
            print_json(serde_json::json!(user_stories), pretty);
        }
        dunno::args::UserStoryCommands::Delete { user_story_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in user_story_ids {
                if db.delete_user_story(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Epic management for project-level feature grouping.
async fn handle_epic_command(
    command: dunno::args::EpicCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::EpicCommands::Create {
            project_id,
            project,
            title,
            description,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;

            let created = db.create_epic(&title, &description, &pid).await?;
            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::EpicCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let epics = match resolved_id {
                Some(pid) => db.list_epics_by_project(&pid).await?,
                None => db.list_epics().await?,
            };
            print_json(serde_json::json!(epics), pretty);
        }
        dunno::args::EpicCommands::Delete { epic_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in epic_ids {
                if db.delete_epic(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Persona management for project-level user personas.
async fn handle_persona_command(
    command: dunno::args::PersonaCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::PersonaCommands::Create {
            project_ids,
            project,
            name,
            content,
        } => {
            let resolved_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-ids/--pids or --project/-p must be provided")
            })?;
            let created = db.create_persona(&name, &content, &pid).await?;
            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::PersonaCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let personas = match resolved_id {
                Some(pid) => db.list_personas_by_project(&pid).await?,
                None => db.list_personas().await?,
            };
            print_json(serde_json::json!(personas), pretty);
        }
        dunno::args::PersonaCommands::Delete { persona_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in persona_ids {
                if db.delete_persona(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Workflow management for project-level process definitions.
async fn handle_workflow_command(
    command: dunno::args::WorkflowCommands,
    db: &dunno::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        dunno::args::WorkflowCommands::Create {
            project_ids,
            project,
            name,
            content,
        } => {
            let resolved_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-ids/--pids or --project/-p must be provided")
            })?;
            let created = db.create_workflow(&name, &content, &pid).await?;
            print_json(serde_json::json!(created), pretty);
        }
        dunno::args::WorkflowCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let workflows = match resolved_id {
                Some(pid) => db.list_workflows_by_project(&pid).await?,
                None => db.list_workflows().await?,
            };
            print_json(serde_json::json!(workflows), pretty);
        }
        dunno::args::WorkflowCommands::Delete { workflow_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in workflow_ids {
                if db.delete_workflow(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            let result = serde_json::json!({
                "status": "ok",
                "deleted": deleted,
                "not_found": not_found,
            });
            print_json(result, pretty);
        }
    }
    Ok(())
}

/// Context gathering for various entity types.
async fn handle_context(
    task_id: Option<String>,
    file_id: Option<String>,
    epic_id: Option<String>,
    full: bool,
    db: &dunno::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match (task_id, file_id, epic_id) {
        (Some(t_id), _, _) => {
            let results = dunno::context::get_task_context(&t_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, Some(f_id), _) => {
            let results = dunno::context::get_file_context(&f_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, _, Some(e_id)) => {
            let results = dunno::context::get_epic_context(&e_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (None, None, None) => {
            return Err(anyhow::anyhow!(
                "One of --task-id, --file-id, or --epic-id must be provided"
            ));
        }
    };
    Ok(())
}

/// Destructive operation to clear all data.
async fn handle_purge(db: &dunno::db::DB, pretty: bool) -> anyhow::Result<()> {
    db.purge_database().await?;
    print_json(
        serde_json::json!({
            "status": "ok",
            "message": "Database purged successfully"
        }),
        pretty,
    );
    Ok(())
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

/// Prints JSON output with optional pretty formatting.
fn print_json(value: serde_json::Value, pretty: bool) {
    if pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        println!("{}", value);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_print_json_compact_format() {
        let value = serde_json::json!({"status": "ok", "id": "task:abc123"});
        let json_str = value.to_string();

        // Compact format should not contain newlines
        assert!(
            !json_str.contains('\n'),
            "compact JSON should not have newlines"
        );
        assert!(
            json_str.contains("status"),
            "JSON should contain field names"
        );
        assert!(
            json_str.contains("task:abc123"),
            "JSON should contain values"
        );
    }

    #[test]
    fn test_print_json_pretty_format() {
        let value = serde_json::json!({"status": "ok", "id": "task:abc123"});
        let pretty_str = serde_json::to_string_pretty(&value).unwrap();

        // Pretty format should contain newlines and indentation
        assert!(
            pretty_str.contains('\n'),
            "pretty JSON should have newlines"
        );
        assert!(
            pretty_str.contains("  "),
            "pretty JSON should have indentation"
        );
    }

    #[test]
    fn test_print_json_handles_nested_objects() {
        let value = serde_json::json!({
            "project": {
                "id": "project:abc",
                "name": "Test"
            },
            "tasks": ["task:1", "task:2"]
        });

        let compact = value.to_string();
        let pretty = serde_json::to_string_pretty(&value).unwrap();

        // Both should parse back to the same value
        let parsed_compact: serde_json::Value = serde_json::from_str(&compact).unwrap();
        let parsed_pretty: serde_json::Value = serde_json::from_str(&pretty).unwrap();

        assert_eq!(parsed_compact, parsed_pretty);
        assert_eq!(parsed_compact["project"]["id"], "project:abc");
    }

    #[test]
    fn test_print_json_handles_arrays() {
        let value = serde_json::json!([
            {"id": "task:1", "name": "Task 1"},
            {"id": "task:2", "name": "Task 2"}
        ]);

        let pretty = serde_json::to_string_pretty(&value).unwrap();

        // Pretty format should have newlines between array items
        assert!(
            pretty.contains('\n'),
            "pretty JSON array should have newlines"
        );
        assert!(
            pretty.contains("Task 1"),
            "pretty JSON should preserve values"
        );
    }
}
