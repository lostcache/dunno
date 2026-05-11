use crate::utils::{print_json, resolve_project_id};

#[derive(clap::Subcommand, Debug)]
pub enum PersonaCommands {
    #[command(
        name = "add",
        about = "Create a new AI agent persona linked to a project."
    )]
    Create {
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            help = "Project ID(s) to link this persona to. Repeatable. Conflicts with --project."
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
        #[arg(help = "Persona name.")]
        name: String,
        #[arg(help = "Persona definition content (tone, rules, behaviour).")]
        content: String,
    },
    #[command(
        name = "list",
        visible_alias = "ls",
        about = "List personas, optionally filtered by project."
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
    #[command(name = "rm", about = "Delete one or more personas by ID.")]
    Delete {
        #[arg(
            required = true,
            help = "One or more persona IDs to delete (e.g. persona:abc)."
        )]
        persona_ids: Vec<String>,
    },
}

pub(crate) async fn handle_persona_command(
    command: PersonaCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        PersonaCommands::Create {
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
            let created = db.create_persona(&name, &content, &pid).await?;
            print_json(serde_json::json!(created), pretty);
        }
        PersonaCommands::List {
            project_id,
            project,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let personas = match resolved_id {
                Some(pid) => db.list_personas_by_project(&pid).await?,
                None => db.list_personas().await?,
            };
            print_json(serde_json::json!(personas), pretty);
        }
        PersonaCommands::Delete { persona_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in persona_ids {
                if db.delete_persona(&id).await? {
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
