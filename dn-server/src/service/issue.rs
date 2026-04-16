use crate::schema;
use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

pub(crate) async fn list_issues(
    Query(q): Query<schema::ListIssuesQuery>,
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let issues = match q.task_id {
        Some(tid) => state.db.list_issues_by_task(&tid).await?,
        None => state.db.list_issues_by_project(&q.project_id).await?,
    };
    Ok(Json(serde_json::to_value(issues)?))
}

pub(crate) async fn create_issue(
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::CreateIssueBody>,
) -> schema::ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_issue(
            &body.description,
            body.task_id.as_deref(),
            body.plan.as_deref(),
            &body.project_id,
            body.verification.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

pub(crate) async fn update_issue(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<schema::UpdateIssueBody>,
) -> schema::ApiResult<serde_json::Value> {
    let status = body
        .status
        .as_deref()
        .and_then(dn_core::models::IssueStatus::parse);
    let updated = state
        .db
        .update_issue(&id, body.description, status, body.plan, body.verification)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

pub(crate) async fn delete_issue(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, schema::ApiError> {
    let deleted = state.db.delete_issue(&id).await?;
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
