use crate::utils::{print_json, resolve_project_id};
use dn_core::db::surreal::DB;

#[derive(clap::Subcommand, Debug)]
pub enum IssueCommands {
    #[command(
        name = "add",
        about = "Create a new issue and optionally link it to a task."
    )]
    Create {
        #[arg(long, visible_alias = "tid", value_name = "TASK_ID")]
        task_id: Option<String>,
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
        #[arg(long, value_name = "PLAN")]
        plan: Option<String>,
        #[arg(long, value_name = "VERIFICATION")]
        verification: Option<String>,
        description: String,
    },
    #[command(
        name = "update",
        about = "Update an existing issue's description, plan, or status."
    )]
    Update {
        issue_id: String,
        #[arg(long, value_name = "DESC")]
        description: Option<String>,
        #[arg(long, value_name = "PLAN")]
        plan: Option<String>,
        #[arg(long, value_name = "VERIFICATION")]
        verification: Option<String>,
        #[arg(
            long,
            value_name = "STATUS",
            help = "One of: pending, active, completed"
        )]
        status: Option<String>,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List issues for a project, optionally filtered by task."
    )]
    List {
        #[arg(long, visible_alias = "pid", value_name = "PROJECT_ID")]
        project_id: String,
        #[arg(long, visible_alias = "tid", value_name = "TASK_ID")]
        task_id: Option<String>,
    },
    #[command(name = "rm", about = "Delete one or more issues by ID.")]
    Remove {
        #[arg(required = true)]
        issue_ids: Vec<String>,
    },
    #[command(about = "Fetch a single issue by ID.")]
    Get {
        #[arg(help = "Issue ID to fetch (e.g. issue:abc).")]
        id: String,
    },
}

pub(crate) async fn handle_issue_command(
    command: IssueCommands,
    db: &DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        IssueCommands::Create {
            task_id,
            project_id,
            project,
            plan,
            verification,
            description,
        } => {
            let resolved_project_id = resolve_project_id(db, project_id, project, ignore_case)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("Either --project-id or --project must be provided")
                })?;
            let created = db
                .create_issue(
                    &description,
                    task_id.as_deref(),
                    plan.as_deref(),
                    &resolved_project_id,
                    verification.as_deref(),
                )
                .await?;
            print_json(serde_json::json!(created), pretty);
        }
        IssueCommands::Update {
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
        IssueCommands::List {
            project_id,
            task_id,
        } => {
            let issues = match task_id {
                Some(tid) => db.list_issues_by_task(&tid).await?,
                None => db.list_issues_by_project(&project_id).await?,
            };
            print_json(serde_json::json!(issues), pretty);
        }
        IssueCommands::Remove { issue_ids } => {
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
        IssueCommands::Get { id } => {
            let issue = db.get_issue(&id).await?;
            match issue {
                Some(i) => print_json(serde_json::json!(i), pretty),
                None => return Err(anyhow::anyhow!("Issue not found: {}", id)),
            }
        }
    }
    Ok(())
}
