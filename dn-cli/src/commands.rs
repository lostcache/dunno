use crate::args;

pub(crate) fn print_json(value: serde_json::Value, pretty: bool) {
    if pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        println!("{}", value);
    }
}

pub(crate) fn print_error_json(kind: &str, message: String) {
    println!(
        "{}",
        serde_json::json!({
            "status": "error",
            "kind": kind,
            "error": message
        })
    );
}

pub(crate) fn validate_task_parents<'a>(
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

pub(crate) fn parse_optional_status(
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

/// Handles config display without requiring database connection.
pub(crate) fn handle_config_command(
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
pub(crate) async fn resolve_project_id(
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
pub(crate) async fn handle_add(
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
pub(crate) async fn handle_rm(
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
pub(crate) async fn handle_link(
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
pub(crate) async fn handle_project_command(
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
pub(crate) async fn handle_module_command(
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
pub(crate) async fn handle_file_command(
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
pub(crate) async fn handle_task_command(
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_print_json_compact_format() {
        let value = serde_json::json!({"status": "ok", "id": "task:abc123"});
        let json_str = value.to_string();

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
