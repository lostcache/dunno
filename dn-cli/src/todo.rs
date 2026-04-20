use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum TodoCommands {
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
        content: String,
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
    #[command(name = "update")]
    Update {
        todo_id: String,
        #[arg(long, value_name = "CONTENT")]
        content: Option<String>,
        #[arg(long, value_name = "STATUS", help = "One of: pending, active, completed")]
        status: Option<String>,
    },
    #[command(name = "rm")]
    Delete {
        #[arg(required = true)]
        todo_ids: Vec<String>,
    },
    Get {
        id: String,
    },
}

pub(crate) async fn handle_todo_command(
    command: TodoCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        TodoCommands::Create {
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
        TodoCommands::List {
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
        TodoCommands::Update {
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
        TodoCommands::Delete { todo_ids } => {
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
        TodoCommands::Get { id } => {
            let todo = db.get_todo(&id).await?;
            match todo {
                Some(t) => print_json(serde_json::json!(t), pretty),
                None => return Err(anyhow::anyhow!("Todo not found: {}", id)),
            }
        }
    }
    Ok(())
}
