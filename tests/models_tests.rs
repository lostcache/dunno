use lazydev::models::{Mistake, Skill, StyleRule};
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
