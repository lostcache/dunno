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
