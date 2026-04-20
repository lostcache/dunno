use crate::commands::print_json;

pub(crate) async fn handle_context(
    task_id: Option<String>,
    file_id: Option<String>,
    epic_id: Option<String>,
    full: bool,
    general: bool,
    project: Option<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    if general {
        let proj =
            project.ok_or_else(|| anyhow::anyhow!("--project / -p is required with --general"))?;
        let project_id = if proj.contains(':') {
            proj.clone()
        } else {
            db.get_project_by_name(&proj, true)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Project not found: {}", proj))?
                .id
                .ok_or_else(|| anyhow::anyhow!("Project has no ID"))?
        };
        let results = dn_core::context::get_project_structure(&project_id, db).await?;
        print_json(
            serde_json::json!({ "results": serde_json::to_value(results)? }),
            pretty,
        );
        return Ok(());
    }

    match (task_id, file_id, epic_id) {
        (Some(t_id), _, _) => {
            let results = dn_core::context::get_task_context(&t_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, Some(f_id), _) => {
            let results = dn_core::context::get_file_context(&f_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (_, _, Some(e_id)) => {
            let results = dn_core::context::get_epic_context(&e_id, full, db).await?;
            print_json(
                serde_json::json!({ "results": serde_json::to_value(results)? }),
                pretty,
            );
        }
        (None, None, None) => {
            return Err(anyhow::anyhow!(
                "One of --task-id, --file-id, --epic-id, or --general must be provided"
            ));
        }
    };
    Ok(())
}

pub(crate) async fn handle_purge(db: &dn_core::db::DB, pretty: bool) -> anyhow::Result<()> {
    db.purge_database().await?;
    print_json(
        serde_json::json!({
            "status": "ok",
            "message": "Database purged successfully"
        }),
        pretty,
    );
    Ok(())
}
