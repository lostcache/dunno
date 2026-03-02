#[derive(Debug)]
pub struct KnowledgeResult {
    pub kind: String,
    pub content: String,
    pub linked: bool,
}

pub async fn add_knowledge(
    kind: String,
    content: String,
    link_to: Vec<String>,
    db: &crate::db::DB,
) -> anyhow::Result<KnowledgeResult> {
    let ctx = crate::models::Context {
        id: None,
        context_type: kind.as_str().to_string(),
        content: Some(content.clone()),
        description: match kind.as_str() {
            "style" => Some(content.clone()),
            _ => None,
        },
        example: match kind.as_str() {
            "style" => Some(String::new()),
            _ => None,
        },
        severity: match kind.as_str() {
            "security" => Some("medium".to_string()),
            _ => None,
        },
        category: match kind.as_str() {
            "security" => Some(String::new()),
            _ => None,
        },
        tags: match kind.as_str() {
            "security" => Some(vec![]),
            _ => None,
        },
    };

    let created = db.create_context(&ctx).await?;
    let record_id = match &created.id {
        Some(id) => id.clone(),
        None => {
            return Ok(KnowledgeResult {
                kind,
                content,
                linked: false,
            });
        }
    };

    for target_id in &link_to {
        db.link_context(target_id, &record_id).await?;
    }

    Ok(KnowledgeResult {
        kind,
        content,
        linked: !link_to.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_add_mistake() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");

        let result = crate::ingest::add_knowledge(
            "mistake".to_string(),
            "Using unwrap".to_string(),
            vec![],
            &db,
        )
        .await
        .expect("add_knowledge should succeed");

        assert_eq!(result.kind, "mistake");
        assert_eq!(result.content, "Using unwrap");
        assert!(!result.linked);

        let contexts = db
            .list_contexts_by_type("mistake")
            .await
            .expect("list_contexts_by_type should succeed");
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].content.as_deref(), Some("Using unwrap"));
    }

    #[tokio::test]
    async fn test_add_knowledge_with_link() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.expect("project id");

        let result = crate::ingest::add_knowledge(
            "mistake".to_string(),
            "Using unwrap".to_string(),
            vec![project_id.clone()],
            &db,
        )
        .await
        .expect("add_knowledge should succeed");

        assert!(result.linked);
    }
}
