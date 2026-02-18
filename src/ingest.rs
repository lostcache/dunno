use anyhow::Result;
use crate::db::DB;
use crate::models::{Mistake, Skill, StyleRule};
use crate::vector_db::VectorDB;

/// Adds a new knowledge record and links it into the graph.
pub async fn add_knowledge(
    category: String,
    kind: String,
    content: String,
    db: &DB,
    _vector_db: &VectorDB,
) -> Result<()> {
    // 1. Generate embedding (Placeholder)
    let _embedding = generate_embedding(&content);

    // 2. Store in SurrealDB based on kind
    let id = match kind.as_str() {
        "mistake" => {
            let mistake = Mistake {
                id: None,
                content: content.clone(),
                category: category.clone(),
                tags: vec![],
            };
            let created = db.create_mistake(&mistake).await?;
            created.id
        },
        "style" => {
            let rule = StyleRule {
                id: None,
                description: content.clone(),
                example: "".to_string(), // TODO: Add example field to CLI
            };
            let created = db.create_style_rule(&rule).await?;
            created.id
        },
        "skill" => {
            let skill = Skill {
                id: None,
                name: content.clone(),
                proficiency: "Basic".to_string(), // Default
            };
            let created = db.create_skill(&skill).await?;
            created.id
        }
        _ => return Err(anyhow::anyhow!("Unknown knowledge type: {}", kind)),
    };

    // 3. Link into the graph via category tags.
    let tag = db.create_or_get_category_tag(&category).await?;
    if let Some(record_id) = id {
        if let Some(tag_id) = tag.id {
            db.create_edge(&record_id, &tag_id, "has_tag").await?;
        }
    }

    Ok(())
}

fn generate_embedding(_text: &str) -> Vec<f32> {
    // Placeholder: Return random vector of size 384 (common small model size)
    vec![0.0; 384]
}
