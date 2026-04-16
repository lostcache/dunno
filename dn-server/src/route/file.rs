use crate::{server::AppState, service};
use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/files", post(service::file::create_file))
        .route(
            "/api/files/:id",
            patch(service::file::update_file).delete(service::file::delete_file),
        )
        .route(
            "/api/modules/:mid/files",
            get(service::file::list_files_by_module),
        )
        .route(
            "/api/projects/:pid/files",
            get(service::file::list_files_by_project),
        )
}
