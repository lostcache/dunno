use crate::{AppState, service};
use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tasks", post(service::task::create_task))
        .route(
            "/api/tasks/:id",
            patch(service::task::update_task).delete(service::task::delete_task),
        )
        .route(
            "/api/projects/:pid/tasks",
            get(service::task::list_tasks_by_project),
        )
}
