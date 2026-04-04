mod args;

use clap::Parser;

/// Application entry point.
#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let args = match args::Args::try_parse() {
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
async fn run(args: args::Args) -> anyhow::Result<()> {
    let config = dn_core::config::Config::load(args.backend.as_deref())?;

    if let args::Commands::Config { command } = &args.command {
        return handle_config_command(command, &config, args.pretty);
    }

    let db = dn_core::db::DB::from_config(&config).await?;
    dispatch_command(args.command, &db, args.pretty, args.ignore_case).await
}

/// Routes commands to their specialized handlers.
async fn dispatch_command(
    command: args::Commands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::Commands::Add {
            field_names,
            field_values,
            link_to,
        } => handle_add(field_names, field_values, link_to, db, pretty).await,
        args::Commands::Link {
            from_id,
            edge,
            to_id,
        } => handle_link(from_id, edge, to_id, db, pretty).await,
        args::Commands::Project { command } => handle_project_command(command, db, pretty).await,
        args::Commands::Module { command } => {
            handle_module_command(command, db, pretty, ignore_case).await
        }
        args::Commands::File { command } => {
            handle_file_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Task { command } => {
            handle_task_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Todo { command } => {
            handle_todo_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Issue { command } => handle_issue_command(command, db, pretty).await,
        args::Commands::UserStory { command } => {
            handle_user_story_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Epic { command } => {
            handle_epic_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Persona { command } => {
            handle_persona_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Workflow { command } => {
            handle_workflow_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Context {
            task_id,
            file_id,
            epic_id,
            full,
            general,
            project,
        } => {
            handle_context(
                task_id, file_id, epic_id, full, general, project, db, pretty,
            )
            .await
        }
        args::Commands::Rm { context_ids } => handle_rm(context_ids, db, pretty).await,
        args::Commands::Purge => handle_purge(db, pretty).await,
        args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}

/// Handles config display without requiring database connection.
fn handle_config_command(
    command: &args::ConfigCommands,
    config: &dn_core::config::Config,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        args::ConfigCommands::Show => {
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
    db: &dn_core::db::DB,
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
    db: &dn_core::db::DB,
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
    dn_core::ingest::add_knowledge_schemaless(map, link_to, db).await?;
    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}

/// Deletes context records by id.
async fn handle_rm(
    context_ids: Vec<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    let mut deleted = Vec::new();
    let mut not_found = Vec::new();
    for id in context_ids {
        if db.delete_context(&id).await? {
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
    Ok(())
}

/// Validates edge types against allowed schema before creating links.
async fn handle_link(
    from_id: Vec<String>,
    edge: Vec<String>,
    to_id: Vec<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    const ALLOWED_EDGES: &[&str] = &[
        "contains",
        "has_file",
        "has_module",
        "has_task",
        "has_todo",
        "has_context",
        "has_user_story",
        "has_epic",
        "has_issue",
        "belongs_to_project",
        "belongs_to_module",
        "belongs_to_task",
        "belongs_to_story",
        "belongs_to_user_story",
        "belongs_to_epic",
    ];

    for e in &edge {
        if !ALLOWED_EDGES.contains(&e.as_str()) {
            return Err(anyhow::anyhow!(
                "Unknown edge {:?}. Allowed: {:?}",
                e,
                ALLOWED_EDGES
            ));
        }
    }

    if to_id.is_empty() {
        return Err(anyhow::anyhow!("At least one --to-id is required"));
    }

    // Single-source mode: one from/edge, one or more to-ids.
    if from_id.len() == 1 && edge.len() == 1 {
        for t in &to_id {
            db.link(&from_id[0], &edge[0], t).await?;
        }
    // Multi-triplet mode: equal counts of from/edge/to.
    } else if from_id.len() == edge.len() && edge.len() == to_id.len() {
        for ((f, e), t) in from_id.iter().zip(edge.iter()).zip(to_id.iter()) {
            db.link(f, e, t).await?;
        }
    } else {
        return Err(anyhow::anyhow!(
            "Mismatched argument counts: --from-id ({}), --edge ({}), --to-id ({}). \
             Either use a single --from-id and --edge with multiple --to-id values, \
             or repeat all three flags the same number of times for multi-triplet mode.",
            from_id.len(),
            edge.len(),
            to_id.len()
        ));
    }

    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}

/// Project management commands.
async fn handle_project_command(
    command: args::ProjectCommands,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        args::ProjectCommands::Create { name, description } => {
            let project = dn_core::models::Project {
                id: None,
                name,
                description,
            };
            let created = db.create_project(&project).await?;
            print_json(serde_json::json!(created), pretty);
        }
        args::ProjectCommands::List => {
            let projects = db.list_projects().await?;
            print_json(serde_json::json!(projects), pretty);
        }
        args::ProjectCommands::Delete { project_ids } => {
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
    command: args::ModuleCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::ModuleCommands::Create {
            project_ids,
            project,
            parent_module_id,
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
                    parent_module_id.as_deref(),
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
        args::ModuleCommands::List {
            project_id,
            project,
            module_id,
        } => {
            let modules = if let Some(mid) = module_id {
                db.list_modules_by_module(&mid).await?
            } else {
                let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
                match resolved_id {
                    Some(pid) => db.list_modules_by_project(&pid).await?,
                    None => db.list_modules().await?,
                }
            };
            print_json(serde_json::json!(modules), pretty);
        }
        args::ModuleCommands::Delete { module_ids } => {
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

/// File management with parent hierarchy linking and project filtering.
async fn handle_file_command(
    command: args::FileCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::FileCommands::Create {
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
        args::FileCommands::List {
            project_id,
            project,
            module_id,
        } => {
            let files = match module_id {
                Some(mid) => db.list_files_by_module(&mid).await?,
                None => {
                    let resolved_id =
                        resolve_project_id(db, project_id, project, ignore_case).await?;
                    match resolved_id {
                        Some(pid) => db.list_files_by_project(&pid).await?,
                        None => db.list_files().await?,
                    }
                }
            };
            print_json(serde_json::json!(files), pretty);
        }
        args::FileCommands::Delete { file_ids } => {
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
    command: args::TaskCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::TaskCommands::Create {
            module_ids,
            project_id,
            project,
            user_story_ids,
            epic_ids,
            name,
            description,
        } => {
            let resolved_project_id =
                resolve_project_id(db, project_id, project, ignore_case).await?;

            let project_ids: Vec<String> = resolved_project_id.into_iter().collect();
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

            print_json(serde_json::json!(created), pretty);
        }
        args::TaskCommands::Update {
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
        args::TaskCommands::List {
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
        args::TaskCommands::Delete { task_ids } => {
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
        args::TaskCommands::Get { id } => {
            let task = db.get_task(&id).await?;
            match task {
                Some(t) => print_json(serde_json::json!(t), pretty),
                None => return Err(anyhow::anyhow!("Task not found: {}", id)),
            }
        }
    }
    Ok(())
}

/// Validates task parent constraints: exactly one project (required), with an optional module.
fn validate_task_parents<'a>(
    module_ids: &'a [String],
    project_ids: &'a [String],
) -> anyhow::Result<(Option<&'a str>, Option<&'a str>)> {
    match (module_ids.len(), project_ids.len()) {
        (0, 1) => Ok((None, Some(&project_ids[0]))),
        (1, 1) => Ok((Some(&module_ids[0]), Some(&project_ids[0]))),
        _ => Err(anyhow::anyhow!(
            "Task create: provide exactly one project ID (with an optional module ID). Got {} module_ids and {} project_ids",
            module_ids.len(),
            project_ids.len()
        )),
    }
}

/// Parses optional status string into typed TaskStatus.
fn parse_optional_status(
    status: Option<String>,
) -> anyhow::Result<Option<dn_core::models::TaskStatus>> {
    match status {
        Some(value) => dn_core::models::TaskStatus::parse(&value)
            .map(Some)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid status '{}'. Expected: pending, active, completed",
                    value
                )
            }),
        None => Ok(None),
    }
}

/// Issue management with optional task linking.
async fn handle_issue_command(
    command: args::IssueCommands,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        args::IssueCommands::Create {
            task_id,
            project_id,
            plan,
            verification,
            description,
        } => {
            let created = db
                .create_issue(
                    &description,
                    task_id.as_deref(),
                    plan.as_deref(),
                    &project_id,
                    verification.as_deref(),
                )
                .await?;
            print_json(serde_json::json!(created), pretty);
        }
        args::IssueCommands::Update {
            issue_id,
            description,
            plan,
            verification,
            status,
        } => {
            let parsed_status = match status.as_deref() {
                Some(s) => {
                    let st = dn_core::models::IssueStatus::parse(s).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid status '{}'. Expected: pending, active, completed",
                            s
                        )
                    })?;
                    Some(st)
                }
                None => None,
            };
            let result = db
                .update_issue(&issue_id, description, parsed_status, plan, verification)
                .await?;
            print_json(serde_json::json!(result), pretty);
        }
        args::IssueCommands::List {
            project_id,
            task_id,
        } => {
            let issues = match task_id {
                Some(tid) => db.list_issues_by_task(&tid).await?,
                None => db.list_issues_by_project(&project_id).await?,
            };
            print_json(serde_json::json!(issues), pretty);
        }
        args::IssueCommands::Remove { issue_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in issue_ids {
                if db.delete_issue(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            print_json(
                serde_json::json!({
                    "status": "ok",
                    "deleted": deleted,
                    "not_found": not_found,
                }),
                pretty,
            );
        }
        args::IssueCommands::Get { id } => {
            let issue = db.get_issue(&id).await?;
            match issue {
                Some(i) => print_json(serde_json::json!(i), pretty),
                None => return Err(anyhow::anyhow!("Issue not found: {}", id)),
            }
        }
    }
    Ok(())
}

/// Todo management with project association.
async fn handle_todo_command(
    command: args::TodoCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::TodoCommands::Create {
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
        args::TodoCommands::List {
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
        args::TodoCommands::Update {
            todo_id,
            content,
            status,
        } => {
            let parsed_status = match status.as_deref() {
                Some(s) => {
                    let st = dn_core::models::TodoStatus::parse(s).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid status '{}'. Expected: pending, active, completed",
                            s
                        )
                    })?;
                    Some(st)
                }
                None => None,
            };
            let result = db.update_todo(&todo_id, content, parsed_status).await?;
            print_json(serde_json::json!(result), pretty);
        }
        args::TodoCommands::Delete { todo_ids } => {
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
        args::TodoCommands::Get { id } => {
            let todo = db.get_todo(&id).await?;
            match todo {
                Some(t) => print_json(serde_json::json!(t), pretty),
                None => return Err(anyhow::anyhow!("Todo not found: {}", id)),
            }
        }
    }
    Ok(())
}

/// User story management with epic linking.
async fn handle_user_story_command(
    command: args::UserStoryCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::UserStoryCommands::Create {
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
        args::UserStoryCommands::List {
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
        args::UserStoryCommands::Delete { user_story_ids } => {
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
        args::UserStoryCommands::Get { id } => {
            let user_story = db.get_user_story(&id).await?;
            match user_story {
                Some(us) => print_json(serde_json::json!(us), pretty),
                None => return Err(anyhow::anyhow!("User story not found: {}", id)),
            }
        }
    }
    Ok(())
}

/// Epic management for project-level feature grouping.
async fn handle_epic_command(
    command: args::EpicCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::EpicCommands::Create {
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
        args::EpicCommands::List {
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
        args::EpicCommands::Delete { epic_ids } => {
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
    command: args::PersonaCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::PersonaCommands::Create {
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
        args::PersonaCommands::List {
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
        args::PersonaCommands::Delete { persona_ids } => {
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
    command: args::WorkflowCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::WorkflowCommands::Create {
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
        args::WorkflowCommands::List {
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
        args::WorkflowCommands::Delete { workflow_ids } => {
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
    general: bool,
    project: Option<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    if general {
        let proj =
            project.ok_or_else(|| anyhow::anyhow!("--project / -p is required with --general"))?;
        let project_id = if proj.contains(':') {
            proj.clone()
        } else {
            db.get_project_by_name(&proj, true)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Project not found: {}", proj))?
                .id
                .ok_or_else(|| anyhow::anyhow!("Project has no ID"))?
        };
        let results = dn_core::context::get_project_structure(&project_id, db).await?;
        print_json(
            serde_json::json!({ "results": serde_json::to_value(results)? }),
            pretty,
        );
        return Ok(());
    }

    match (task_id, file_id, epic_id) {
        (Some(t_id), _, _) => {
            let results = dn_core::context::get_task_context(&t_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, Some(f_id), _) => {
            let results = dn_core::context::get_file_context(&f_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, _, Some(e_id)) => {
            let results = dn_core::context::get_epic_context(&e_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (None, None, None) => {
            return Err(anyhow::anyhow!(
                "One of --task-id, --file-id, --epic-id, or --general must be provided"
            ));
        }
    };
    Ok(())
}

/// Destructive operation to clear all data.
async fn handle_purge(db: &dn_core::db::DB, pretty: bool) -> anyhow::Result<()> {
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
