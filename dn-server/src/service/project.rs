use crate::schema;
use crate::{
    AppState,
    schema::{CreateProjectBody, UpdateProjectBody},
};
use axum::{Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

pub async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let projects = state.db.list_projects().await?;
    Ok(Json(serde_json::to_value(projects)?))
}

pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateProjectBody>,
) -> schema::ApiResult<serde_json::Value> {
    let project = dn_core::models::Project {
        id: None,
        name: body.name,
        description: body.description,
    };
    let created = state.db.create_project(&project).await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub async fn delete_project(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_project(&id).await?;
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

pub async fn update_project(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProjectBody>,
) -> schema::ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_project(&id, body.name, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}
