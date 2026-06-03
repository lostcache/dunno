use crate::utils::{print_json, resolve_project_id};
use dn_core::db::surreal::DB;

#[derive(clap::Subcommand, Debug)]
pub enum FileCommands {
    #[command(
        name = "add",
        about = "Register a file node and link it to a project and optional parent module."
    )]
    Add {
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            help = "Project ID(s) to link this file to. Repeatable. Conflicts with --project.",
            conflicts_with = "project"
        )]
        project_ids: Vec<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            help = "Project name (resolved to ID). Conflicts with --project-ids.",
            conflicts_with = "project_ids"
        )]
        project: Option<String>,
        #[arg(
            long,
            value_name = "PARENT_ID",
            help = "Parent module ID(s) that contain this file. Repeatable."
        )]
        parent_ids: Vec<String>,
        #[arg(help = "Display name for the file.")]
        name: String,
        #[arg(help = "Relative path to the file (e.g. src/main.rs).")]
        path: String,
        #[arg(
            value_name = "DESCRIPTION",
            help = "Optional short description of the file's purpose."
        )]
        description: Option<String>,
        #[arg(
            long,
            value_name = "NOTES",
            help = "Additional notes attached to this file."
        )]
        notes: Option<String>,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List files, optionally filtered by project or module."
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
        #[arg(long, visible_alias = "mid", help = "Filter by module ID.")]
        module_id: Option<String>,
    },
    #[command(name = "rm", about = "Delete one or more file nodes by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more file IDs to delete (e.g. file:abc)."
        )]
        file_ids: Vec<String>,
    },
}

pub(crate) async fn handle_file_command(
    command: FileCommands,
    db: &DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        FileCommands::Add {
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
