use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mistake {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
    pub category: String,
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
