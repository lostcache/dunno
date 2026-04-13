use crate::{AppState, service};
use axum::{
    Router,
    routing::{get, patch},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/projects",
            get(service::project::list_projects).post(service::project::create_project),
        )
        .route(
            "/api/projects/:id",
            patch(service::project::update_project).delete(service::project::delete_project),
        )
}
