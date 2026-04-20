use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum EpicCommands {
    #[command(name = "add")]
    Create {
        #[arg(
            long,
            visible_alias = "pid",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        title: String,
        description: String,
    },
    #[command(name = "list", visible_alias = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
    },
    #[command(name = "rm")]
    Delete {
        #[arg(required = true)]
        epic_ids: Vec<String>,
    },
}

pub(crate) async fn handle_epic_command(
    command: EpicCommands,
    db: &dn_core::db::DB,
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
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;

            let created = db.create_epic(&title, &description, &pid).await?;
            print_json(serde_json::json!(created), pretty);
        }
        EpicCommands::List {
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
