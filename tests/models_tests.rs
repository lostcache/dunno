use lazydev::models::{CategoryTag, KnowledgeEdge, Mistake, Skill, StyleRule};
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
