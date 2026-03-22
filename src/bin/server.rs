use axum::Json;
use axum::body::Body;
use axum::http::{Method, Response, header, header::CONTENT_TYPE};
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use clap::Parser;
use dunno::{config::Config, db::surreal::DB};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(RustEmbed)]
#[folder = "static/dist/"]
struct StaticFiles;

fn serve_embedded(path: &str) -> Option<Response<Body>> {
    let file = StaticFiles::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(file.data.into_owned()))
        .ok()
}

async fn static_handler(uri: axum::http::Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    serve_embedded(path).unwrap_or_else(|| {
        // SPA fallback — serve index.html for client-side routes
        serve_embedded("index.html").unwrap_or_else(|| {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("UI not built — run `make ui-build`"))
                .unwrap()
        })
    })
}

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 7700)]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    backend: Option<String>,
}

struct AppState {
    db: DB,
}

struct ApiError(anyhow::Error);

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({ "error": self.0.to_string() });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Deserialize)]
struct FullQuery {
    full: Option<bool>,
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateProjectBody {
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct CreateModuleBody {
    name: String,
    description: String,
    notes: Option<String>,
    project_id: String,
}

#[derive(Deserialize)]
struct CreateSubmoduleBody {
    name: String,
    description: String,
    notes: Option<String>,
    module_id: String,
    project_id: String,
}

#[derive(Deserialize)]
struct CreateFileBody {
    name: String,
    path: String,
    description: Option<String>,
    notes: Option<String>,
    project_id: String,
    parent_id: Option<String>,
}

#[derive(Deserialize)]
struct CreateTaskBody {
    name: String,
    description: String,
    module_id: Option<String>,
    project_id: Option<String>,
}

#[derive(Deserialize)]
struct UpdateTaskBody {
    name: Option<String>,
    description: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct CreateTodoBody {
    content: String,
    project_id: Option<String>,
}

#[derive(Deserialize)]
struct CreateUserStoryBody {
    title: String,
    description: String,
    project_id: String,
}

#[derive(Deserialize)]
struct CreateEpicBody {
    title: String,
    description: String,
    project_id: String,
}

#[derive(Deserialize)]
struct CreatePersonaBody {
    name: String,
    content: String,
    project_id: String,
}

#[derive(Deserialize)]
struct CreateWorkflowBody {
    name: String,
    content: String,
    project_id: String,
}

#[derive(Deserialize)]
struct CreateContextBody {
    fields: serde_json::Map<String, serde_json::Value>,
    link_to: String,
}

#[derive(Deserialize)]
struct UpdateProjectBody {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct UpdateModuleBody {
    name: Option<String>,
    description: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateSubmoduleBody {
    name: Option<String>,
    description: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateFileBody {
    name: Option<String>,
    path: Option<String>,
    description: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateUserStoryBody {
    title: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct UpdateEpicBody {
    title: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct UpdateTodoBody {
    content: Option<String>,
}

#[derive(Deserialize)]
struct UpdatePersonaBody {
    name: Option<String>,
    content: Option<String>,
}

#[derive(Deserialize)]
struct UpdateWorkflowBody {
    name: Option<String>,
    content: Option<String>,
}

#[derive(Deserialize)]
struct UpdateContextBody {
    fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct LinkBody {
    from_id: String,
    edge: String,
    to_id: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

// Projects
async fn list_projects(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    let projects = state.db.list_projects().await?;
    Ok(Json(serde_json::to_value(projects)?))
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateProjectBody>,
) -> ApiResult<serde_json::Value> {
    let project = dunno::models::Project {
        id: None,
        name: body.name,
        description: body.description,
    };
    let created = state.db.create_project(&project).await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_project(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_project(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_project(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProjectBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_project(&id, body.name, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Modules
async fn list_modules(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    let modules = state.db.list_modules().await?;
    Ok(Json(serde_json::to_value(modules)?))
}

async fn list_modules_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let modules = state.db.list_modules_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(modules)?))
}

async fn create_module(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateModuleBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_module(
            &body.name,
            &body.description,
            body.notes.as_deref(),
            &body.project_id,
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_module(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_module(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_module(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateModuleBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_module(&id, body.name, body.description, body.notes)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Submodules
async fn list_submodules_by_module(
    Path(mid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let submodules = state.db.list_submodules_by_module(&mid).await?;
    Ok(Json(serde_json::to_value(submodules)?))
}

async fn create_submodule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSubmoduleBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_submodule(
            &body.name,
            &body.description,
            body.notes.as_deref(),
            &body.module_id,
            &body.project_id,
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_submodule(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_submodule(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_submodule(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateSubmoduleBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_submodule(&id, body.name, body.description, body.notes)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Files
async fn list_files_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let files = state.db.list_files_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(files)?))
}

async fn list_files_by_module(
    Path(mid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let files = state.db.list_files_by_module(&mid).await?;
    Ok(Json(serde_json::to_value(files)?))
}

async fn create_file(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateFileBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_file(
            &body.name,
            &body.path,
            body.description.as_deref(),
            body.notes.as_deref(),
            &body.project_id,
            body.parent_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_file(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateFileBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_file(&id, body.name, body.path, body.description, body.notes)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Tasks
async fn list_tasks_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let tasks = state.db.list_tasks_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(tasks)?))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTaskBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_task(
            &body.name,
            &body.description,
            body.module_id.as_deref(),
            body.project_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn update_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateTaskBody>,
) -> ApiResult<serde_json::Value> {
    let status = body
        .status
        .as_deref()
        .and_then(dunno::models::TaskStatus::parse);
    let updated = state
        .db
        .update_task(&id, body.name, body.description, status)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

async fn delete_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_task(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

// Todos
async fn list_todos_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let todos = state.db.list_todos_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(todos)?))
}

async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTodoBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_todo(&body.content, body.project_id.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_todo(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_todo(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_todo(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateTodoBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state.db.update_todo(&id, body.content).await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// User stories
async fn list_user_stories_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let stories = state.db.list_user_stories_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(stories)?))
}

async fn create_user_story(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUserStoryBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_user_story(&body.title, &body.description, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_user_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_user_story(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_user_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateUserStoryBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_user_story(&id, body.title, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Epics
async fn list_epics_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let epics = state.db.list_epics_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(epics)?))
}

async fn create_epic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateEpicBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_epic(&body.title, &body.description, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_epic(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_epic(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_epic(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateEpicBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_epic(&id, body.title, body.description)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Personas
async fn list_personas_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let personas = state.db.list_personas_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(personas)?))
}

async fn create_persona(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePersonaBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_persona(&body.name, &body.content, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_persona(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_persona(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_persona(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdatePersonaBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_persona(&id, body.name, body.content)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Workflows
async fn list_workflows_by_project(
    Path(pid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let workflows = state.db.list_workflows_by_project(&pid).await?;
    Ok(Json(serde_json::to_value(workflows)?))
}

async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateWorkflowBody>,
) -> ApiResult<serde_json::Value> {
    let created = state
        .db
        .create_workflow(&body.name, &body.content, &body.project_id)
        .await?;
    Ok(Json(serde_json::to_value(created)?))
}

async fn delete_workflow(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.db.delete_workflow(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"not found"})),
        )
            .into_response())
    }
}

async fn update_workflow(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateWorkflowBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state
        .db
        .update_workflow(&id, body.name, body.content)
        .await?;
    Ok(Json(serde_json::to_value(updated)?))
}

// Contexts
async fn create_context(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateContextBody>,
) -> ApiResult<serde_json::Value> {
    let created = state.db.create_context_schemaless(body.fields).await?;
    let ctx_id = created
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("context created without id"))?
        .to_string();
    state.db.link_context(&body.link_to, &ctx_id).await?;
    Ok(Json(created))
}

async fn update_context(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateContextBody>,
) -> ApiResult<serde_json::Value> {
    let updated = state.db.update_context(&id, body.fields).await?;
    Ok(Json(updated))
}

// Context queries
async fn get_task_context(
    Path(id): Path<String>,
    Query(q): Query<FullQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_task_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}

async fn get_file_context(
    Path(id): Path<String>,
    Query(q): Query<FullQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_file_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}

async fn get_epic_context(
    Path(id): Path<String>,
    Query(q): Query<FullQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let ctx = state
        .db
        .get_epic_context(&id, q.full.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(ctx)?))
}

// Link
async fn link_nodes(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LinkBody>,
) -> ApiResult<serde_json::Value> {
    state
        .db
        .link(&body.from_id, &body.edge, &body.to_id)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// Graph
async fn get_graph(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    let data = state.db.get_graph_data().await?;
    Ok(Json(data))
}

async fn get_project_graph(
    State(state): State<Arc<AppState>>,
    Path(pid): Path<String>,
) -> ApiResult<serde_json::Value> {
    let data = state.db.get_graph_data_by_project(&pid).await?;
    Ok(Json(data))
}

// ── Router ────────────────────────────────────────────────────────────────────

fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        // Projects
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/:id",
            patch(update_project).delete(delete_project),
        )
        .route("/api/projects/:pid/modules", get(list_modules_by_project))
        .route("/api/projects/:pid/files", get(list_files_by_project))
        .route("/api/projects/:pid/tasks", get(list_tasks_by_project))
        .route("/api/projects/:pid/todos", get(list_todos_by_project))
        .route(
            "/api/projects/:pid/user-stories",
            get(list_user_stories_by_project),
        )
        .route("/api/projects/:pid/epics", get(list_epics_by_project))
        .route("/api/projects/:pid/personas", get(list_personas_by_project))
        .route(
            "/api/projects/:pid/workflows",
            get(list_workflows_by_project),
        )
        // Modules
        .route("/api/modules", get(list_modules).post(create_module))
        .route(
            "/api/modules/:id",
            patch(update_module).delete(delete_module),
        )
        .route(
            "/api/modules/:mid/submodules",
            get(list_submodules_by_module),
        )
        .route("/api/modules/:mid/files", get(list_files_by_module))
        // Submodules
        .route("/api/submodules", post(create_submodule))
        .route(
            "/api/submodules/:id",
            patch(update_submodule).delete(delete_submodule),
        )
        // Files
        .route("/api/files", post(create_file))
        .route("/api/files/:id", patch(update_file).delete(delete_file))
        // Tasks
        .route("/api/tasks", post(create_task))
        .route("/api/tasks/:id", patch(update_task).delete(delete_task))
        // Todos
        .route("/api/todos", post(create_todo))
        .route("/api/todos/:id", patch(update_todo).delete(delete_todo))
        // User stories
        .route("/api/user-stories", post(create_user_story))
        .route(
            "/api/user-stories/:id",
            patch(update_user_story).delete(delete_user_story),
        )
        // Epics
        .route("/api/epics", post(create_epic))
        .route("/api/epics/:id", patch(update_epic).delete(delete_epic))
        // Personas
        .route("/api/personas", post(create_persona))
        .route(
            "/api/personas/:id",
            patch(update_persona).delete(delete_persona),
        )
        // Workflows
        .route("/api/workflows", post(create_workflow))
        .route(
            "/api/workflows/:id",
            patch(update_workflow).delete(delete_workflow),
        )
        // Context
        .route("/api/contexts", post(create_context))
        .route("/api/contexts/:id", patch(update_context))
        .route("/api/ctx/task/:id", get(get_task_context))
        .route("/api/ctx/file/:id", get(get_file_context))
        .route("/api/ctx/epic/:id", get(get_epic_context))
        // Link
        .route("/api/link", post(link_nodes))
        // Graph
        .route("/api/graph", get(get_graph))
        .route("/api/projects/:pid/graph", get(get_project_graph))
        .fallback(static_handler)
        .with_state(state)
        .layer(cors)
}

/// Resolves to () when Ctrl+C is received.
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}

fn find_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind :0");
    listener.local_addr().unwrap().port()
}

async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    false
}

fn spawn_surreal_server(
    surreal_port: u16,
    db_path: &std::path::Path,
) -> anyhow::Result<std::process::Child> {
    std::process::Command::new("surreal")
        .args([
            "start",
            "--bind",
            &format!("127.0.0.1:{}", surreal_port),
            "--username",
            "root",
            "--password",
            "root",
            &format!("surrealkv://{}", db_path.to_string_lossy()),
        ])
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "surreal binary not found — install it with:\n  curl -sSf https://install.surrealdb.com | sh"
                )
            } else {
                e.into()
            }
        })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let config = Config::load(args.backend.as_deref())?;

    let (db, surreal_child): (DB, Option<std::process::Child>) =
        if matches!(config.backend, dunno::config::StorageBackend::Local) {
            let db_path = config.local_data_path();
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let surreal_port = find_free_port();
            match spawn_surreal_server(surreal_port, &db_path) {
                Ok(child) => {
                    if !wait_for_port(surreal_port, 10).await {
                        return Err(anyhow::anyhow!(
                            "surreal server did not start within 10 seconds"
                        ));
                    }
                    let ws_url = format!("ws://127.0.0.1:{}/rpc", surreal_port);
                    let db = DB::new(&ws_url).await?;
                    (db, Some(child))
                }
                Err(e) => {
                    eprintln!("Warning: {e}");
                    eprintln!("dn-server started, but the dn CLI will not be available concurrently.");
                    eprintln!("To use both at the same time, either:");
                    eprintln!("  - Install surreal and restart dn-server: curl -sSf https://install.surrealdb.com | sh");
                    eprintln!("  - Or set backend = \"dev\" in your config and run SurrealDB separately");
                    let db = DB::from_config(&config).await?;
                    (db, None)
                }
            }
        } else {
            let db = DB::from_config(&config).await?;
            (db, None)
        };

    let state = Arc::new(AppState { db });

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let url = format!("http://{}", addr);
    println!("dn-server running at {}", url);

    if !args.no_open {
        open::that(&url).ok();
    }

    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if let Some(mut child) = surreal_child {
        let _ = child.kill();
    }

    Ok(())
}
