use crate::{server::AppState, service};
use axum::{Router, routing::post};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/link", post(service::link::link_nodes))
}
