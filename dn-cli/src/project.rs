use crate::utils::print_json;

#[derive(clap::Subcommand, Debug)]
pub enum ProjectCommands {
    #[command(name = "add")]
    Create { name: String, description: String },
    #[command(name = "list", visible_alias = "ls")]
    List,
    #[command(name = "rm")]
    Delete {
        #[arg(required = true)]
        project_ids: Vec<String>,
    },
}

pub(crate) async fn handle_project_command(
    command: ProjectCommands,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        ProjectCommands::Create { name, description } => {
            let project = dn_core::models::Project {
                id: None,
                name,
                description,
            };
            let created = db.create_project(&project).await?;
            print_json(serde_json::json!(created), pretty);
        }
        ProjectCommands::List => {
            let projects = db.list_projects().await?;
            print_json(serde_json::json!(projects), pretty);
        }
        ProjectCommands::Delete { project_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in project_ids {
                if db.delete_project(&id).await? {
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
