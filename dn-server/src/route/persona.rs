use crate::{AppState, service};
use axum::{Router, routing::{get, patch, post}};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/personas", post(service::persona::create_persona))
        .route(
            "/api/personas/:id",
            patch(service::persona::update_persona).delete(service::persona::delete_persona),
        )
        .route(
            "/api/projects/:pid/personas",
            get(service::persona::list_personas_by_project),
        )
}
