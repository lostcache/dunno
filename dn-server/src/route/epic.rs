use crate::{server::AppState, service};
use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/epics", post(service::epic::create_epic))
        .route(
            "/api/epics/:id",
            patch(service::epic::update_epic).delete(service::epic::delete_epic),
        )
        .route(
            "/api/projects/:pid/epics",
            get(service::epic::list_epics_by_project),
        )
}
