use crate::db::DB;
use crate::models::{Mistake, SecurityDetail, StyleRule};
use anyhow::Result;

/// Adds a new knowledge record and optionally links it to a structural node.
pub async fn add_knowledge(
    kind: String,
    content: String,
    link_to: Option<String>,
    db: &DB,
) -> Result<()> {
    let id = match kind.as_str() {
        "mistake" => {
            let mistake = Mistake {
                id: None,
                content: content.clone(),
            };
            let created = db.create_mistake(&mistake).await?;
            created.id
        }
        "style" => {
            let rule = StyleRule {
                id: None,
                description: content.clone(),
                example: String::new(),
            };
            let created = db.create_style_rule(&rule).await?;
            created.id
        }
        "security" => {
            let detail = SecurityDetail {
                id: None,
                content: content.clone(),
                severity: "medium".to_string(),
                category: String::new(),
                tags: vec![],
            };
            let created = db.create_security_detail(&detail).await?;
            created.id
        }
        _ => return Err(anyhow::anyhow!("Unknown knowledge type: {}", kind)),
    };

    // Link to a structural node (Project/Module/Task/etc.) if requested.
    if let (Some(target_id), Some(record_id)) = (link_to, &id) {
        db.link_context(&target_id, record_id).await?;
    }

    Ok(())
}
