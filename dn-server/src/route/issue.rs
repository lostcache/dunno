use crate::{AppState, service};
use axum::{Router, routing::{get, patch}};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/issues",
            get(service::issue::list_issues).post(service::issue::create_issue),
        )
        .route(
            "/api/issues/:id",
            patch(service::issue::update_issue).delete(service::issue::delete_issue),
        )
}
