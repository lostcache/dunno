use crate::{AppState, service};
use axum::{Router, routing::get};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/graph", get(service::graph::get_graph))
        .route(
            "/api/projects/:pid/graph",
            get(service::graph::get_project_graph),
        )
}
