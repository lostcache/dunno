use crate::utils::{parse_optional_status, print_json, resolve_project_id, validate_task_parents};

#[derive(clap::Subcommand, Debug)]
pub enum TaskCommands {
    #[command(
        name = "add",
        about = "Create a new task linked to a project, and optionally to modules, user stories, or epics."
    )]
    Create {
        #[arg(
            long,
            visible_alias = "mids",
            value_name = "MODULE_ID",
            help = "Module ID(s) to associate with this task. Repeatable."
        )]
        module_ids: Vec<String>,
        #[arg(
            long,
            visible_alias = "pid",
            value_name = "PROJECT_ID",
            help = "Project ID. Required unless --project is provided.",
            conflicts_with = "project",
            required_unless_present = "project"
        )]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            help = "Project name (resolved to ID). Required unless --project-id is provided.",
            conflicts_with = "project_id",
            required_unless_present = "project_id"
        )]
        project: Option<String>,
        #[arg(
            long,
            visible_alias = "usids",
            value_name = "USER_STORY_ID",
            help = "User story ID(s) to link this task to. Repeatable."
        )]
        user_story_ids: Vec<String>,
        #[arg(
            long,
            visible_alias = "eids",
            value_name = "EPIC_ID",
            help = "Epic ID(s) to link this task to. Repeatable."
        )]
        epic_ids: Vec<String>,
        #[arg(help = "Task name.")]
        name: String,
        #[arg(help = "Full task description or implementation plan.")]
        description: String,
    },
    #[command(about = "Update a task's name, description, or status.")]
    Update {
        #[arg(help = "Task ID to update (e.g. task:abc).")]
        task_id: String,
        #[arg(long, help = "New task name.")]
        name: Option<String>,
        #[arg(long, help = "New task description.")]
        description: Option<String>,
        #[arg(
            long,
            value_name = "STATUS",
            help = "One of: pending, active, completed"
        )]
        status: Option<String>,
    },
    #[command(name = "rm", about = "Delete one or more tasks by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more task IDs to delete (e.g. task:abc)."
        )]
        task_ids: Vec<String>,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List tasks, optionally filtered by project."
    )]
    List {
        #[arg(
            long,
            visible_alias = "pid",
            help = "Filter by project ID. Conflicts with --project.",
            conflicts_with = "project"
        )]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            help = "Filter by project name. Conflicts with --project-id.",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
    },
    #[command(about = "Fetch a single task by ID.")]
    Get {
        #[arg(help = "Task ID to fetch (e.g. task:abc).")]
        id: String,
    },
}

pub(crate) async fn handle_task_command(
    command: TaskCommands,
    db: &dn_core::db::surreal::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        TaskCommands::Create {
            module_ids,
            project_id,
            project,
            user_story_ids,
            epic_ids,
            name,
            description,
        } => {
            if project.is_none() && project_id.is_none() {
                anyhow::bail!("Either --project-id or --project must be provided");
            }
            let resolved_project_id = match project_id {
                Some(id) => id,
                None => resolve_project_id(db, project.unwrap(), ignore_case).await?,
            };

            let project_ids: Vec<String> = vec![resolved_project_id];
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
        TaskCommands::Update {
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
        TaskCommands::List {
            project_id,
            project,
        } => {
            if project.is_none() && project_id.is_none() {
                anyhow::bail!("Either --project-id or --project must be provided");
            }
            let resolved_id = match project_id {
                Some(id) => id,
                None => resolve_project_id(db, project.unwrap(), ignore_case).await?,
            };
            let tasks = db.list_tasks_by_project(&resolved_id).await?;
            print_json(serde_json::json!(tasks), pretty);
        }
        TaskCommands::Delete { task_ids } => {
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
        TaskCommands::Get { id } => {
            let task = db.get_task(&id).await?;
            match task {
                Some(t) => print_json(serde_json::json!(t), pretty),
                None => return Err(anyhow::anyhow!("Task not found: {}", id)),
            }
        }
    }
    Ok(())
}
