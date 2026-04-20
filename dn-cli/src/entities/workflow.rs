use crate::args;
use crate::commands::{print_json, resolve_project_id};

pub(crate) async fn handle_workflow_command(
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
