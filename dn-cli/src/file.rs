use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum FileCommands {
    #[command(name = "add")]
    Create {
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_ids: Vec<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_ids"
        )]
        project: Option<String>,
        #[arg(long, value_name = "PARENT_ID")]
        parent_ids: Vec<String>,
        name: String,
        path: String,
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
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
        #[arg(long, visible_alias = "mid")]
        module_id: Option<String>,
    },
    #[command(name = "rm")]
    Delete {
        #[arg(required = true)]
        file_ids: Vec<String>,
    },
}

pub(crate) async fn handle_file_command(
    command: FileCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        FileCommands::Create {
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
        FileCommands::List {
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
        FileCommands::Delete { file_ids } => {
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
