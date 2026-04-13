use crate::schema;
use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_epics_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let epics = state.db.list_epics_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(epics)?))
}

pub(crate) async fn create_epic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateEpicBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_epic(&body.title, &body.description, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_epic(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateEpicBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_epic(&id, body.title, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_epic(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_epic(&id).await?;
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
