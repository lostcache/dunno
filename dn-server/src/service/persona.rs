use crate::schema;
use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_personas_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let personas = state.db.list_personas_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(personas)?))
}

pub(crate) async fn create_persona(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreatePersonaBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_persona(&body.name, &body.content, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_persona(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdatePersonaBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_persona(&id, body.name, body.content)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_persona(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_persona(&id).await?;
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
