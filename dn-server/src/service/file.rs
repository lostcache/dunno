use crate::schema;
use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_files_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let files = state.db.list_files_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(files)?))
}

pub(crate) async fn list_files_by_module(
    Path(mid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let files = state.db.list_files_by_module(&mid).await?;
    Ok(Json(serde_json::to_value(files)?))
}

pub(crate) async fn create_file(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateFileBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_file(
            &body.name,
            &body.path,
            body.description.as_deref(),
            body.notes.as_deref(),
            &body.project_id,
            body.parent_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateFileBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_file(&id, body.name, body.path, body.description, body.notes)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_file(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}
