use crate::AppState;
use crate::schema;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_modules(
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let modules = state.db.list_modules().await?;
    Ok(Json(serde_json::to_value(modules)?))
}

pub(crate) async fn list_modules_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let modules = state.db.list_modules_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(modules)?))
}

pub(crate) async fn list_modules_by_module(
    Path(mid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let modules = state.db.list_modules_by_module(&mid).await?;
    Ok(Json(serde_json::to_value(modules)?))
}

pub(crate) async fn create_module(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateModuleBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_module(
            &body.name,
            &body.description,
            body.notes.as_deref(),
            &body.project_id,
            body.parent_module_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_module(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateModuleBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_module(&id, body.name, body.description, body.notes)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_module(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_module(&id).await?;
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
