#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserStory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Epic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Persona {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Workflow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    Started,
    Finished,
}

impl TaskStatus {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "not_started" => Some(Self::NotStarted),
            "started" => Some(Self::Started),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Module {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Submodule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct File {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TodoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Default)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Free-form context type (e.g. mistake, style_rule, security_detail, code_styleguide, skill).
    #[serde(rename = "type")]
    pub context_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SubmoduleInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TaskHierarchy {
    pub project_id: String,
    pub project_name: String,
    pub module_id: Option<String>,
    pub module_name: Option<String>,
    pub submodule: Option<SubmoduleInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TaskContext {
    pub persona: Vec<Persona>,
    pub workflow: Vec<Workflow>,
    pub task: Task,
    pub files: Vec<File>,
    pub contexts: Vec<Context>,
    pub hierarchy: TaskHierarchy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EpicContext {
    pub persona: Vec<Persona>,
    pub workflow: Vec<Workflow>,
    pub epic: Epic,
    pub contexts: Vec<Context>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProjectContext {
    pub project: Project,
    pub contexts: Vec<Context>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SubmoduleStructure {
    pub submodule: Submodule,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ModuleStructure {
    pub module: Module,
    pub submodules: Vec<SubmoduleStructure>,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProjectStructure {
    pub project: Project,
    pub modules: Vec<ModuleStructure>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ModuleContext {
    pub module: Module,
    pub contexts: Vec<Context>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SubmoduleContext {
    pub submodule: Submodule,
    pub contexts: Vec<Context>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FileContext {
    pub persona: Vec<Persona>,
    pub workflow: Vec<Workflow>,
    pub file: File,
    pub contexts: Vec<Context>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn task_status_parse_accepts_known_values() {
        assert_eq!(
            TaskStatus::parse("not_started"),
            Some(TaskStatus::NotStarted)
        );
        assert_eq!(TaskStatus::parse("started"), Some(TaskStatus::Started));
        assert_eq!(TaskStatus::parse("finished"), Some(TaskStatus::Finished));
    }

    #[test]
    fn task_status_parse_rejects_unknown_values() {
        assert_eq!(TaskStatus::parse("in_progress"), None);
        assert_eq!(TaskStatus::parse(""), None);
        assert_eq!(TaskStatus::parse("NOT_STARTED"), None);
    }

    #[test]
    fn test_project_model() {
        let project = Project {
            id: None,
            name: "My Project".to_string(),
            description: "A description".to_string(),
        };
        let json = to_string(&project).expect("Failed to serialize Project");
        assert!(json.contains("My Project"));
    }

    #[test]
    fn test_module_model() {
        let module = Module {
            id: None,
            name: "Core".to_string(),
            description: "Core module".to_string(),
            notes: None,
        };
        let json = to_string(&module).expect("Failed to serialize Module");
        assert!(json.contains("Core module"));
    }

    #[test]
    fn test_submodule_model() {
        let submodule = Submodule {
            id: None,
            name: "Lexer".to_string(),
            description: "Lexer submodule".to_string(),
            notes: None,
        };
        let json = to_string(&submodule).expect("Failed to serialize Submodule");
        assert!(json.contains("Lexer submodule"));
    }

    #[test]
    fn test_file_model() {
        let file = File {
            id: None,
            name: "lexer.rs".to_string(),
            path: "src/lexer.rs".to_string(),
            description: Some("Lexer implementation for tokenizing input".to_string()),
            notes: None,
        };
        let json = to_string(&file).expect("Failed to serialize File");
        assert!(json.contains("lexer.rs"));
    }

    #[test]
    fn test_task_model() {
        let task = Task {
            id: None,
            name: "Implement Auth".to_string(),
            description: "Add login".to_string(),
            status: TaskStatus::NotStarted,
        };
        let json = to_string(&task).expect("Failed to serialize Task");
        assert!(json.contains("Implement Auth"));
        assert!(json.contains("\"status\":\"not_started\""));
    }

    #[test]
    fn test_todo_model() {
        let todo = TodoItem {
            id: None,
            content: "Fix bug".to_string(),
        };
        let json = to_string(&todo).expect("Failed to serialize TodoItem");
        assert!(json.contains("Fix bug"));
    }

    #[test]
    fn test_context_model() {
        let ctx = Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Avoid unwrap in production code".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: Some(vec!["error-handling".to_string()]),
        };
        let json = to_string(&ctx).expect("Failed to serialize Context");
        assert!(json.contains("\"type\":\"mistake\""));
        assert!(json.contains("Avoid unwrap in production code"));
    }

    #[test]
    fn test_user_story_model() {
        let user_story = UserStory {
            id: None,
            title: "As a user, I want...".to_string(),
            description: "User should be able to login".to_string(),
        };
        let json = to_string(&user_story).expect("Failed to serialize UserStory");
        assert!(json.contains("As a user, I want..."));
        assert!(json.contains("User should be able to login"));
    }

    #[test]
    fn test_epic_model() {
        let epic = Epic {
            id: None,
            title: "Authentication Epic".to_string(),
            description: "Implement complete authentication system".to_string(),
        };
        let json = to_string(&epic).expect("Failed to serialize Epic");
        assert!(json.contains("Authentication Epic"));
        assert!(json.contains("Implement complete authentication system"));
    }

    #[test]
    fn test_persona_model() {
        let persona = Persona {
            id: None,
            name: "Senior Rust Developer".to_string(),
            content: "You are a senior Rust developer who prefers functional patterns.".to_string(),
        };
        let json = to_string(&persona).expect("Failed to serialize Persona");
        assert!(json.contains("Senior Rust Developer"));
        assert!(json.contains("functional patterns"));
    }

    #[test]
    fn test_workflow_model() {
        let workflow = Workflow {
            id: None,
            name: "TDD Workflow".to_string(),
            content: "Write tests first, then implementation, then docs.".to_string(),
        };
        let json = to_string(&workflow).expect("Failed to serialize Workflow");
        assert!(json.contains("TDD Workflow"));
        assert!(json.contains("Write tests first"));
    }

    #[test]
    fn test_task_context_persona_workflow_serialized_first() {
        let task_ctx = TaskContext {
            persona: vec![Persona {
                id: None,
                name: "Dev".to_string(),
                content: "persona content".to_string(),
            }],
            workflow: vec![Workflow {
                id: None,
                name: "Flow".to_string(),
                content: "workflow content".to_string(),
            }],
            task: Task {
                id: None,
                name: "t".to_string(),
                description: "d".to_string(),
                status: TaskStatus::NotStarted,
            },
            files: vec![],
            contexts: vec![],
            hierarchy: TaskHierarchy {
                project_id: "project:1".to_string(),
                project_name: "proj".to_string(),
                module_id: Some("module:1".to_string()),
                module_name: Some("mod".to_string()),
                submodule: None,
            },
        };
        let json = to_string(&task_ctx).expect("serialize");
        let persona_pos = json.find("\"persona\"").expect("persona key present");
        let workflow_pos = json.find("\"workflow\"").expect("workflow key present");
        let task_pos = json.find("\"task\"").expect("task key present");
        assert!(persona_pos < task_pos, "persona must appear before task");
        assert!(workflow_pos < task_pos, "workflow must appear before task");
    }

    #[test]
    fn test_epic_context_persona_workflow_serialized_first() {
        let epic_ctx = EpicContext {
            persona: vec![Persona {
                id: None,
                name: "Dev".to_string(),
                content: "persona content".to_string(),
            }],
            workflow: vec![],
            epic: Epic {
                id: None,
                title: "Auth".to_string(),
                description: "d".to_string(),
            },
            contexts: vec![],
        };
        let json = to_string(&epic_ctx).expect("serialize");
        let persona_pos = json.find("\"persona\"").expect("persona key present");
        let epic_pos = json.find("\"epic\"").expect("epic key present");
        assert!(persona_pos < epic_pos, "persona must appear before epic");
    }

    #[test]
    fn test_file_context_persona_workflow_serialized_first() {
        let file_ctx = FileContext {
            persona: vec![],
            workflow: vec![Workflow {
                id: None,
                name: "Flow".to_string(),
                content: "workflow content".to_string(),
            }],
            file: File {
                id: None,
                name: "main.rs".to_string(),
                path: "src/main.rs".to_string(),
                description: None,
                notes: None,
            },
            contexts: vec![],
        };
        let json = to_string(&file_ctx).expect("serialize");
        let workflow_pos = json.find("\"workflow\"").expect("workflow key present");
        let file_pos = json.find("\"file\"").expect("file key present");
        assert!(workflow_pos < file_pos, "workflow must appear before file");
    }
}
