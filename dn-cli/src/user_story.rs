use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum UserStoryCommands {
    #[command(
        name = "add",
        about = "Create a new user story linked to a project and optional epic(s)."
    )]
    Create {
        #[arg(
            long,
            visible_alias = "pid",
            value_name = "PROJECT_ID",
            help = "Project ID. Conflicts with --project.",
            conflicts_with = "project"
        )]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            help = "Project name (resolved to ID). Conflicts with --project-id.",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        #[arg(
            long,
            visible_alias = "eids",
            value_name = "EPIC_ID",
            help = "Epic ID(s) to link this user story to. Repeatable."
        )]
        epic_ids: Vec<String>,
        #[arg(help = "User story title.")]
        title: String,
        #[arg(help = "User story description.")]
        description: String,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List user stories, optionally filtered by project or epic."
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
        #[arg(
            long,
            visible_alias = "eid",
            value_name = "EPIC_ID",
            help = "Filter by epic ID."
        )]
        epic_id: Option<String>,
    },
    #[command(name = "rm", about = "Delete one or more user stories by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more user story IDs to delete (e.g. user_story:abc)."
        )]
        user_story_ids: Vec<String>,
    },
    #[command(about = "Fetch a single user story by ID.")]
    Get {
        #[arg(help = "User story ID to fetch (e.g. user_story:abc).")]
        id: String,
    },
}

pub(crate) async fn handle_user_story_command(
    command: UserStoryCommands,
    db: &dn_core::db::surreal::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        UserStoryCommands::Create {
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
        UserStoryCommands::List {
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
        UserStoryCommands::Delete { user_story_ids } => {
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
        UserStoryCommands::Get { id } => {
            let user_story = db.get_user_story(&id).await?;
            match user_story {
                Some(us) => print_json(serde_json::json!(us), pretty),
                None => return Err(anyhow::anyhow!("User story not found: {}", id)),
            }
        }
    }
    Ok(())
}
