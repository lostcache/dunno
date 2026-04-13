use crate::{AppState, service};
use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/user-stories",
            post(service::user_story::create_user_story),
        )
        .route(
            "/api/user-stories/:id",
            patch(service::user_story::update_user_story)
                .delete(service::user_story::delete_user_story),
        )
        .route(
            "/api/projects/:pid/user-stories",
            get(service::user_story::list_user_stories_by_project),
        )
}
