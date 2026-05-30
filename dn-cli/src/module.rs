use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum ModuleCommands {
    #[command(name = "add", about = "Create a new module and link it to a project.")]
    Create {
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            help = "Project ID(s) to link this module to. Repeatable. Conflicts with --project.",
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
            visible_alias = "pmid",
            value_name = "PARENT_MODULE_ID",
            help = "Parent module ID for nested modules."
        )]
        parent_module_id: Option<String>,
        #[arg(help = "Module name.")]
        name: String,
        #[arg(help = "Short description of the module.")]
        description: String,
        #[arg(
            long,
            value_name = "NOTES",
            help = "Additional notes attached to this module."
        )]
        notes: Option<String>,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List modules, optionally filtered by project or parent module."
    )]
    List {
        #[arg(long, visible_alias = "pid", help = "Filter by project ID. Conflicts with --project and --module-id.", conflicts_with_all = ["project", "module_id"])]
        project_id: Option<String>,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            help = "Filter by project name. Conflicts with --project-id and --module-id.",
            conflicts_with_all = ["project_id", "module_id"]
        )]
        project: Option<String>,
        #[arg(long, visible_alias = "mid", help = "List child modules of this parent module ID. Conflicts with --project-id and --project.", conflicts_with_all = ["project_id", "project"])]
        module_id: Option<String>,
    },
    #[command(name = "rm", about = "Delete one or more modules by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more module IDs to delete (e.g. module:abc)."
        )]
        module_ids: Vec<String>,
    },
}

pub(crate) async fn handle_module_command(
    command: ModuleCommands,
    db: &dn_core::db::surreal::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        ModuleCommands::Create {
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

            for pid in project_ids.iter().skip(1) {
                db.link(pid, "contains", module_id).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        ModuleCommands::List {
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
        ModuleCommands::Delete { module_ids } => {
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
