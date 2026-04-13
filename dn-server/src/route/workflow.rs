use crate::{AppState, service};
use axum::{Router, routing::{get, patch, post}};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/workflows", post(service::workflow::create_workflow))
        .route(
            "/api/workflows/:id",
            patch(service::workflow::update_workflow).delete(service::workflow::delete_workflow),
        )
        .route(
            "/api/projects/:pid/workflows",
            get(service::workflow::list_workflows_by_project),
        )
}
