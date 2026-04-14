use crate::{AppState, route};
use axum::{
    Router,
    http::{Method, header::CONTENT_TYPE},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .merge(route::project::routes())
        .merge(route::module::routes())
        .merge(route::file::routes())
        .merge(route::task::routes())
        .merge(route::todo::routes())
        .merge(route::user_story::routes())
        .merge(route::epic::routes())
        .merge(route::persona::routes())
        .merge(route::workflow::routes())
        .merge(route::context::routes())
        .merge(route::issue::routes())
        .merge(route::link::routes())
        .merge(route::graph::routes())
        .fallback(crate::ui::static_handler)
        .with_state(state)
        .layer(cors)
}
