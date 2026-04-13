use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use serde::Deserialize;

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

pub(crate) struct ApiError(anyhow::Error);

pub(crate) type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Deserialize)]
pub(crate) struct CreateProjectBody {
    pub(crate) name: String,
    pub(crate) description: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateModuleBody {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) notes: Option<String>,
    pub(crate) project_id: String,
    pub(crate) parent_module_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CreateFileBody {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) description: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) project_id: String,
    pub(crate) parent_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CreateTaskBody {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) module_id: Option<String>,
    pub(crate) project_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateTaskBody {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CreateTodoBody {
    pub(crate) content: String,
    pub(crate) project_id: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateUserStoryBody {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) project_id: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateEpicBody {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) project_id: String,
}

#[derive(Deserialize)]
pub(crate) struct CreatePersonaBody {
    pub(crate) name: String,
    pub(crate) content: String,
    pub(crate) project_id: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateWorkflowBody {
    pub(crate) name: String,
    pub(crate) content: String,
    pub(crate) project_id: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateContextBody {
    pub(crate) fields: serde_json::Map<String, serde_json::Value>,
    pub(crate) link_to: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateProjectBody {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateModuleBody {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) notes: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateFileBody {
    pub(crate) name: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) notes: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateUserStoryBody {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateEpicBody {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateTodoBody {
    pub(crate) content: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdatePersonaBody {
    pub(crate) name: Option<String>,
    pub(crate) content: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateWorkflowBody {
    pub(crate) name: Option<String>,
    pub(crate) content: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateContextBody {
    pub(crate) fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub(crate) struct ListIssuesQuery {
    pub(crate) project_id: String,
    pub(crate) task_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CreateIssueBody {
    pub(crate) description: String,
    pub(crate) task_id: Option<String>,
    pub(crate) project_id: String,
    pub(crate) plan: Option<String>,
    pub(crate) verification: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateIssueBody {
    pub(crate) description: Option<String>,
    pub(crate) plan: Option<String>,
    pub(crate) verification: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LinkBody {
    pub(crate) from_id: String,
    pub(crate) edge: String,
    pub(crate) to_id: String,
}

#[derive(Deserialize)]
pub(crate) struct FullQuery {
    pub(crate) full: Option<bool>,
}
