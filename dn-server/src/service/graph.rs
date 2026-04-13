use crate::AppState;
use crate::schema;
use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;

pub(crate) async fn get_graph(
    State(state): State<Arc<AppState>>,
) -> schema::ApiResult<serde_json::Value> {
    let data = state.db.get_graph_data().await?;
    Ok(Json(data))
}

pub(crate) async fn get_project_graph(
    State(state): State<Arc<AppState>>,
    Path(pid): Path<String>,
) -> schema::ApiResult<serde_json::Value> {
    let data = state.db.get_graph_data_by_project(&pid).await?;
    Ok(Json(data))
}
