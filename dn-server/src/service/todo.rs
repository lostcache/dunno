use crate::{schema, server::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_todos_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let todos = state.db.list_todos_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(todos)?))
}

pub(crate) async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateTodoBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_todo(&body.content, Some(&body.project_id))
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_todo(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateTodoBody>,
) -> schema::ApiResult<serde_json::Value> {
    let parsed_status = match body.status.as_deref() {
        Some(s) => {
            let st = dn_core::models::TodoStatus::parse(s).ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid status '{}'. Expected: pending, active, completed",
                    s
                )
            })?;
            Some(st)
        }
        None => None,
    };
    let updated = state
        .db
        .update_todo(&id, body.content, parsed_status)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_todo(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_todo(&id).await?;
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
