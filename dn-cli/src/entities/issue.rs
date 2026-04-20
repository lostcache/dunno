use crate::args;
use crate::commands::print_json;

pub(crate) async fn handle_issue_command(
    command: args::IssueCommands,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        args::IssueCommands::Create {
            task_id,
            project_id,
            plan,
            verification,
            description,
        } => {
            let created = db
                .create_issue(
                    &description,
                    task_id.as_deref(),
                    plan.as_deref(),
                    &project_id,
                    verification.as_deref(),
                )
                .await?;
            print_json(serde_json::json!(created), pretty);
        }
        args::IssueCommands::Update {
            issue_id,
            description,
            plan,
            verification,
            status,
        } => {
            let parsed_status = match status.as_deref() {
                Some(s) => {
                    let st = dn_core::models::IssueStatus::parse(s).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid status '{}'. Expected: pending, active, completed",
                            s
                        )
                    })?;
                    Some(st)
                }
                None => None,
            };
            let result = db
                .update_issue(&issue_id, description, parsed_status, plan, verification)
                .await?;
            print_json(serde_json::json!(result), pretty);
        }
        args::IssueCommands::List {
            project_id,
            task_id,
        } => {
            let issues = match task_id {
                Some(tid) => db.list_issues_by_task(&tid).await?,
                None => db.list_issues_by_project(&project_id).await?,
            };
            print_json(serde_json::json!(issues), pretty);
        }
        args::IssueCommands::Remove { issue_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in issue_ids {
                if db.delete_issue(&id).await? {
                    deleted.push(id);
                } else {
                    not_found.push(id);
                }
            }
            print_json(
                serde_json::json!({
                    "status": "ok",
                    "deleted": deleted,
                    "not_found": not_found,
                }),
                pretty,
            );
        }
        args::IssueCommands::Get { id } => {
            let issue = db.get_issue(&id).await?;
            match issue {
                Some(i) => print_json(serde_json::json!(i), pretty),
                None => return Err(anyhow::anyhow!("Issue not found: {}", id)),
            }
        }
    }
    Ok(())
}
