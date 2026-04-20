use crate::args;
use crate::commands::{print_json, resolve_project_id};

pub(crate) async fn handle_persona_command(
    command: args::PersonaCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::PersonaCommands::Create {
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
        args::PersonaCommands::List {
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
        args::PersonaCommands::Delete { persona_ids } => {
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
