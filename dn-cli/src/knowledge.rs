use crate::utils::print_json;

pub(crate) async fn handle_add(
    field_names: Vec<String>,
    field_values: Vec<String>,
    link_to: Vec<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    if field_names.len() != field_values.len() {
        return Err(anyhow::anyhow!(
            "Number of --field flags ({}) must match number of --value flags ({})",
            field_names.len(),
            field_values.len()
        ));
    }

    let mut map = serde_json::Map::new();
    for (key, value) in field_names.into_iter().zip(field_values.into_iter()) {
        map.insert(key, serde_json::Value::String(value));
    }
    dn_core::ingest::add_knowledge_schemaless(map, link_to, db).await?;
    print_json(serde_json::json!({ "status": "ok" }), pretty);
    Ok(())
}

pub(crate) async fn handle_rm(
    context_ids: Vec<String>,
    db: &dn_core::db::DB,
    pretty: bool,
) -> anyhow::Result<()> {
    let mut deleted = Vec::new();
    let mut not_found = Vec::new();
    for id in context_ids {
        if db.delete_context(&id).await? {
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
    Ok(())
}
