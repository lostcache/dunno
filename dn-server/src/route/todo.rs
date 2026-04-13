use crate::{AppState, service};
use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/todos", post(service::todo::create_todo))
        .route(
            "/api/todos/:id",
            patch(service::todo::update_todo).delete(service::todo::delete_todo),
        )
        .route(
            "/api/projects/:pid/todos",
            get(service::todo::list_todos_by_project),
        )
}
