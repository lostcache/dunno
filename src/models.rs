use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub project_id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub module_id: String,
    pub name: String,
    pub description: String,
    pub status: String, // e.g., "pending", "in_progress", "done"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub project_id: String,
    pub task_id: Option<String>, // Optional link to a specific task context
    pub content: String,
    pub status: String, // e.g., "pending", "claimed", "completed"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mistake {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
    pub category: String, // Can be used for tagging, but primary link is via graph
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub proficiency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryTag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub normalized: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn test_mistake_model() {
        let mistake = Mistake {
            id: None,
            content: "Using unwrap instead of expect".to_string(),
            category: "rust".to_string(),
            tags: vec!["error-handling".to_string()],
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
    fn test_skill_model() {
        let skill = Skill {
            id: None,
            name: "Async Rust".to_string(),
            proficiency: "Intermediate".to_string(),
        };

        let json = to_string(&skill).expect("Failed to serialize Skill");
        assert!(json.contains("Async Rust"));
    }

    #[test]
    fn test_category_tag_model() {
        let tag = CategoryTag {
            id: None,
            name: "Rust".to_string(),
            normalized: "rust".to_string(),
        };

        let json = to_string(&tag).expect("Failed to serialize CategoryTag");
        assert!(json.contains("\"normalized\":\"rust\""));
    }

    #[test]
    fn test_knowledge_edge_model() {
        let edge = KnowledgeEdge {
            id: None,
            from_id: "mistake:1".to_string(),
            to_id: "category_tag:rust".to_string(),
            relation: "has_tag".to_string(),
        };

        let json = to_string(&edge).expect("Failed to serialize KnowledgeEdge");
        assert!(json.contains("\"relation\":\"has_tag\""));
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
            project_id: "project:1".to_string(),
            name: "Core".to_string(),
            description: "Core module".to_string(),
        };
        let json = to_string(&module).expect("Failed to serialize Module");
        assert!(json.contains("Core module"));
    }

    #[test]
    fn test_task_model() {
        let task = Task {
            id: None,
            module_id: "module:1".to_string(),
            name: "Implement Auth".to_string(),
            description: "Add login".to_string(),
            status: "pending".to_string(),
        };
        let json = to_string(&task).expect("Failed to serialize Task");
        assert!(json.contains("Implement Auth"));
    }

    #[test]
    fn test_todo_model() {
        let todo = TodoItem {
            id: None,
            project_id: "project:1".to_string(),
            task_id: Some("task:1".to_string()),
            content: "Fix bug".to_string(),
            status: "pending".to_string(),
        };
        let json = to_string(&todo).expect("Failed to serialize TodoItem");
        assert!(json.contains("Fix bug"));
    }
}
