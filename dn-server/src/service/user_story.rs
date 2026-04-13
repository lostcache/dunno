use crate::AppState;
use crate::schema;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_user_stories_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let stories = state.db.list_user_stories_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(stories)?))
}

pub(crate) async fn create_user_story(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateUserStoryBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_user_story(&body.title, &body.description, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_user_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateUserStoryBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_user_story(&id, body.title, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_user_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_user_story(&id).await?;
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
