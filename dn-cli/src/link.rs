use crate::utils::print_json;
use dn_core::db::surreal::DB;

pub(crate) async fn handle_link(
    from_id: Vec<String>,
    edge: Vec<String>,
    to_id: Vec<String>,
    db: &DB,
    pretty: bool,
) -> anyhow::Result<()> {
    const ALLOWED_EDGES: &[&str] = &[
        "contains",
        "has_file",
        "has_module",
        "has_task",
        "has_todo",
        "has_context",
        "has_user_story",
        "has_epic",
        "has_issue",
        "belongs_to_project",
        "belongs_to_module",
        "belongs_to_task",
        "belongs_to_story",
        "belongs_to_user_story",
        "belongs_to_epic",
    ];

    for e in &edge {
        if !ALLOWED_EDGES.contains(&e.as_str()) {
            return Err(anyhow::anyhow!(
                "Unknown edge {:?}. Allowed: {:?}",
                e,
                ALLOWED_EDGES
            ));
        }
    }

    if to_id.is_empty() {
        return Err(anyhow::anyhow!("At least one --to-id is required"));
    }

    // Single-source mode: one from/edge, one or more to-ids.
    if from_id.len() == 1 && edge.len() == 1 {
        for t in &to_id {
            db.link(&from_id[0], &edge[0], t).await?;
        }
    // Multi-triplet mode: equal counts of from/edge/to.
    } else if from_id.len() == edge.len() && edge.len() == to_id.len() {
        for ((f, e), t) in from_id.iter().zip(edge.iter()).zip(to_id.iter()) {
            db.link(f, e, t).await?;
        }
    } else {
        return Err(anyhow::anyhow!(
            "Mismatched argument counts: --from-id ({}), --edge ({}), --to-id ({}). \
             Either use a single --from-id and --edge with multiple --to-id values, \
             or repeat all three flags the same number of times for multi-triplet mode.",
            from_id.len(),
            edge.len(),
            to_id.len()
        ));
    }

    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}
