use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum WorkflowCommands {
    #[command(
        name = "add",
        about = "Create a new workflow definition linked to a project."
    )]
    Create {
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            help = "Project ID(s) to link this workflow to. Repeatable. Conflicts with --project."
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
        #[arg(help = "Workflow name.")]
        name: String,
        #[arg(help = "Workflow definition content (steps, rules, instructions).")]
        content: String,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List workflows, optionally filtered by project."
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
    #[command(name = "rm", about = "Delete one or more workflows by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more workflow IDs to delete (e.g. workflow:abc)."
        )]
        workflow_ids: Vec<String>,
    },
}

pub(crate) async fn handle_workflow_command(
    command: WorkflowCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        WorkflowCommands::Create {
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
        WorkflowCommands::List {
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
        WorkflowCommands::Delete { workflow_ids } => {
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
