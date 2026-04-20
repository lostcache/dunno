use crate::args;
use crate::commands::{print_json, resolve_project_id};

pub(crate) async fn handle_todo_command(
    command: args::TodoCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::TodoCommands::Create {
            project_ids,
            project,
            content,
        } => {
            let resolved_project_id =
                resolve_project_id(db, project_ids.first().cloned(), project, ignore_case).await?;

            let created = db
                .create_todo(&content, resolved_project_id.as_deref())
                .await?;

            let todo_id = match &created.id {
                Some(id) => id.as_str(),
                None => {
                    print_json(serde_json::json!(created), pretty);
                    return Ok(());
                }
            };

            for pid in project_ids.iter().skip(1) {
                db.link(pid, "has_todo", todo_id).await?;
            }

            print_json(serde_json::json!(created), pretty);
        }
        args::TodoCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;
            let todos = db.list_todos_by_project(&pid).await?;
            print_json(serde_json::json!(todos), pretty);
        }
        args::TodoCommands::Update {
            todo_id,
            content,
            status,
        } => {
            let parsed_status = match status.as_deref() {
                Some(s) => {
                    let st = dn_core::models::TodoStatus::parse(s).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid status '{}'. Expected: pending, active, completed",
                            s
                        )
                    })?;
                    Some(st)
                }
                None => None,
            };
            let result = db.update_todo(&todo_id, content, parsed_status).await?;
            print_json(serde_json::json!(result), pretty);
        }
        args::TodoCommands::Delete { todo_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in todo_ids {
                if db.delete_todo(&id).await? {
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
        args::TodoCommands::Get { id } => {
            let todo = db.get_todo(&id).await?;
            match todo {
                Some(t) => print_json(serde_json::json!(t), pretty),
                None => return Err(anyhow::anyhow!("Todo not found: {}", id)),
            }
        }
    }
    Ok(())
}
