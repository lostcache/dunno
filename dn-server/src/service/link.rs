use crate::schema;
use crate::AppState;
use axum::{Json, extract::State};
use std::sync::Arc;

pub(crate) async fn link_nodes(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::LinkBody>,
) -> schema::ApiResult<serde_json::Value> {
    state
        .db
        .link(&body.from_id, &body.edge, &body.to_id)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
