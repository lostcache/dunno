use crate::schema;
use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_tasks_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let tasks = state.db.list_tasks_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(tasks)?))
}

pub(crate) async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateTaskBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_task(
            &body.name,
            &body.description,
            body.module_id.as_deref(),
            body.project_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateTaskBody>,
) -> schema::ApiResult<serde_json::Value> {
    let status = body
        .status
        .as_deref()
        .and_then(dn_core::models::TaskStatus::parse);
    let updated = state
        .db
        .update_task(&id, body.name, body.description, status)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_task(&id).await?;
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
