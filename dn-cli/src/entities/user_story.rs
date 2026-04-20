use crate::args;
use crate::commands::{print_json, resolve_project_id};

pub(crate) async fn handle_user_story_command(
    command: args::UserStoryCommands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::UserStoryCommands::Create {
            project_id,
            project,
            epic_ids,
            title,
            description,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let pid = resolved_id.ok_or_else(|| {
                anyhow::anyhow!("Either --project-id or --project must be provided")
            })?;

            let created = db.create_user_story(&title, &description, &pid).await?;

            if let Some(us_id) = &created.id {
                for epic_id in &epic_ids {
                    db.link_user_story_to_epic(us_id, epic_id).await?;
                }
            }

            print_json(serde_json::json!(created), pretty);
        }
        args::UserStoryCommands::List {
            project_id,
            project,
            epic_id,
        } => {
            let resolved_id = resolve_project_id(db, project_id, project, ignore_case).await?;
            let user_stories = match (epic_id, resolved_id) {
                (Some(eid), _) => db.list_user_stories_by_epic(&eid).await?,
                (_, Some(pid)) => db.list_user_stories_by_project(&pid).await?,
                (None, None) => db.list_user_stories().await?,
            };
            print_json(serde_json::json!(user_stories), pretty);
        }
        args::UserStoryCommands::Delete { user_story_ids } => {
            let mut deleted = Vec::new();
            let mut not_found = Vec::new();
            for id in user_story_ids {
                if db.delete_user_story(&id).await? {
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
        args::UserStoryCommands::Get { id } => {
            let user_story = db.get_user_story(&id).await?;
            match user_story {
                Some(us) => print_json(serde_json::json!(us), pretty),
                None => return Err(anyhow::anyhow!("User story not found: {}", id)),
            }
        }
    }
    Ok(())
}
