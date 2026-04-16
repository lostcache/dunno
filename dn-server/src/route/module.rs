use crate::{server::AppState, service};
use axum::{
    Router,
    routing::{get, patch},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/modules",
            get(service::module::list_modules).post(service::module::create_module),
        )
        .route(
            "/api/modules/:id",
            patch(service::module::update_module).delete(service::module::delete_module),
        )
        .route(
            "/api/modules/:mid/modules",
            get(service::module::list_modules_by_module),
        )
        .route(
            "/api/projects/:pid/modules",
            get(service::module::list_modules_by_project),
        )
}
