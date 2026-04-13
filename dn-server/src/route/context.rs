use crate::{AppState, service};
use axum::{Router, routing::{get, patch, post}};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/contexts", post(service::context::create_context))
        .route(
            "/api/contexts/:id",
            patch(service::context::update_context).delete(service::context::delete_context),
        )
        .route("/api/ctx/task/:id", get(service::context::get_task_context))
        .route("/api/ctx/file/:id", get(service::context::get_file_context))
        .route("/api/ctx/epic/:id", get(service::context::get_epic_context))
}
