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
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Submodule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct File {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub path: String,
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
pub struct Subtask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TaskUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
    pub created_at_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TodoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Mistake {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct StyleRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SecurityDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
    pub severity: String,
    pub category: String,
    pub tags: Vec<String>,
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
    pub module_id: String,
    pub module_name: String,
    pub submodule: Option<SubmoduleInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TaskContext {
    pub task: Task,
    pub subtasks: Vec<Subtask>,
    pub updates: Vec<TaskUpdate>,
    pub files: Vec<String>,
    pub mistakes: Vec<Mistake>,
    pub style_rules: Vec<StyleRule>,
    pub security_details: Vec<SecurityDetail>,
    pub hierarchy: TaskHierarchy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

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
            files: None,
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
            files: None,
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
    fn test_subtask_model() {
        let subtask = Subtask {
            id: None,
            name: "Write unit tests".to_string(),
            description: "Tests for login flow".to_string(),
            status: TaskStatus::NotStarted,
        };
        let json = to_string(&subtask).expect("Failed to serialize Subtask");
        assert!(json.contains("Write unit tests"));
        assert!(json.contains("\"status\":\"not_started\""));
    }

    #[test]
    fn test_task_update_model() {
        let update = TaskUpdate {
            id: None,
            content: "Implemented initial endpoint wiring".to_string(),
            created_at_ms: 1_739_000_000_000,
            updated_at_ms: None,
        };
        let json = to_string(&update).expect("Failed to serialize TaskUpdate");
        assert!(json.contains("endpoint wiring"));
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
    fn test_mistake_model() {
        let mistake = Mistake {
            id: None,
            content: "Using unwrap instead of expect".to_string(),
        };
        let json = to_string(&mistake).expect("Failed to serialize Mistake");
        assert!(json.contains("Using unwrap instead of expect"));
    }

    #[test]
    fn test_style_rule_model() {
        let rule = StyleRule {
            id: None,
            description: "Prefer functional style for iterators".to_string(),
            example: "vec.iter().map(...).collect()".to_string(),
        };
        let json = to_string(&rule).expect("Failed to serialize StyleRule");
        assert!(json.contains("Prefer functional style"));
    }

    #[test]
    fn test_security_detail_model() {
        let detail = SecurityDetail {
            id: None,
            content: "SQL injection risk in raw queries".to_string(),
            severity: "high".to_string(),
            category: "injection".to_string(),
            tags: vec!["sql".to_string(), "security".to_string()],
        };
        let json = to_string(&detail).expect("Failed to serialize SecurityDetail");
        assert!(json.contains("SQL injection risk"));
        assert!(json.contains("\"severity\":\"high\""));
    }
}
