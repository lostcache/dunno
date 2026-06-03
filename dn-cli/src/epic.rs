use crate::utils::{print_json, resolve_project_id};
use dn_core::db::surreal::DB;

#[derive(clap::Subcommand, Debug)]
pub enum EpicCommands {
    #[command(name = "add", about = "Create a new epic linked to a project.")]
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
        #[arg(help = "Epic title.")]
        title: String,
        #[arg(help = "Epic description.")]
        description: String,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List epics, optionally filtered by project."
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
    #[command(name = "rm", about = "Delete one or more epics by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more epic IDs to delete (e.g. epic:abc)."
        )]
        epic_ids: Vec<String>,
    },
}

pub(crate) async fn handle_epic_command(
    command: EpicCommands,
    db: &DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        EpicCommands::Create {
            project_id,
            project,
            title,
            description,
        } => {
            if project.is_none() && project_id.is_none() {
                anyhow::bail!("Either --project-id or --project must be provided");
            }

            let resolved_id = match project_id {
                Some(id) => id,
                None => resolve_project_id(db, project.unwrap(), ignore_case).await?,
            };
            let created = db.create_epic(&title, &description, &resolved_id).await?;
            print_json(serde_json::json!(created), pretty);
        }
        EpicCommands::List {
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
            let epics = db.list_epics_by_project(&resolved_id).await?;
            print_json(serde_json::json!(epics), pretty);
        }
        EpicCommands::Delete { epic_ids } => {
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
