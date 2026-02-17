use anyhow::Result;
use crate::models::{Mistake, StyleRule, Skill};
use crate::db::DB;
use crate::vector_db::VectorDB;

pub async fn add_knowledge(
    category: String,
    kind: String,
    content: String,
    db: &DB,
    _vector_db: &VectorDB
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
        },
        _ => return Err(anyhow::anyhow!("Unknown knowledge type: {}", kind)),
    };
    
    // 3. Store in Qdrant
    if let Some(record_id) = id {
        // TODO: Store embedding + record_id in Qdrant
        // vector_db.upsert(embedding, record_id).await?;
        println!("Would store embedding for ID: {}", record_id);
    }
    
    Ok(())
}

fn generate_embedding(_text: &str) -> Vec<f32> {
    // Placeholder: Return random vector of size 384 (common small model size)
    vec![0.0; 384]
}
