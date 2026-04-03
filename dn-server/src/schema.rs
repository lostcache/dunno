use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectBody {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct CreateModuleBody {
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub project_id: String,
    pub parent_module_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateFileBody {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub project_id: String,
    pub parent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTaskBody {
    pub name: String,
    pub description: String,
    pub module_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTaskBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTodoBody {
    pub content: String,
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct CreateUserStoryBody {
    pub title: String,
    pub description: String,
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct CreateEpicBody {
    pub title: String,
    pub description: String,
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct CreatePersonaBody {
    pub name: String,
    pub content: String,
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct CreateWorkflowBody {
    pub name: String,
    pub content: String,
    pub project_id: String,
}

#[derive(Deserialize)]
pub struct CreateContextBody {
    pub fields: serde_json::Map<String, serde_json::Value>,
    pub link_to: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectBody {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateModuleBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateFileBody {
    pub name: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserStoryBody {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateEpicBody {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTodoBody {
    pub content: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePersonaBody {
    pub name: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowBody {
    pub name: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateContextBody {
    pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ListIssuesQuery {
    pub project_id: String,
    pub task_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateIssueBody {
    pub description: String,
    pub task_id: Option<String>,
    pub project_id: String,
    pub plan: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateIssueBody {
    pub description: Option<String>,
    pub plan: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct LinkBody {
    pub from_id: String,
    pub edge: String,
    pub to_id: String,
}

#[derive(Deserialize)]
pub struct FullQuery {
    pub full: Option<bool>,
}
