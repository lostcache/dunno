use crate::AppState;
use crate::schema;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn create_context(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateContextBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state.db.create_context_schemaless(body.fields).await?;
    let ctx_id = created
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("context created without id"))?
        .to_string();
    state.db.link_context(&body.link_to, &ctx_id).await?;
    Ok(Json(created))
}

pub(crate) async fn update_context(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateContextBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state.db.update_context(&id, body.fields).await?;
    Ok(Json(updated))
}

pub(crate) async fn delete_context(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_context(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "context not found" })),
        )
            .into_response())
    }
}

pub(crate) async fn get_task_context(
    Path(id): Path<String>,
    Query(q): Query<schema::FullQuery>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_task_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}

pub(crate) async fn get_file_context(
    Path(id): Path<String>,
    Query(q): Query<schema::FullQuery>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_file_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}

pub(crate) async fn get_epic_context(
    Path(id): Path<String>,
    Query(q): Query<schema::FullQuery>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_epic_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}
