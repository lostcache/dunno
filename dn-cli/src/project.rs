use crate::utils::print_json;

#[derive(clap::Subcommand, Debug)]
pub enum ProjectCommands {
    #[command(name = "add", about = "Create a new project.")]
    Create {
        #[arg(help = "Project name.")]
        name: String,
        #[arg(help = "Short description of the project.")]
        description: String,
    },
    #[command(name = "list", visible_alias = "ls", about = "List all projects.")]
    List,
    #[command(name = "rm", about = "Delete one or more projects by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more project IDs to delete (e.g. project:abc)."
        )]
        project_ids: Vec<String>,
    },
}

pub(crate) async fn handle_project_command(
    command: ProjectCommands,
    db: &dn_core::db::surreal::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    match command {
        ProjectCommands::Create { name, description } => {
            let project = dn_core::models::Project {
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
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
