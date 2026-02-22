
#[derive(Debug)]
pub struct KnowledgeResult {
    pub kind: String,
    pub content: String,
    pub linked: bool,
}

/// Adds a new knowledge record and optionally links it to a structural node.
pub async fn add_knowledge(
    kind: String,
    content: String,
    link_to: Option<String>,
    db: &crate::db::DB,
) -> anyhow::Result<KnowledgeResult> {
    let id = match kind.as_str() {
        "mistake" => {
            let mistake = crate::models::Mistake {
                id: None,
                content: content.clone(),
            };
            let created = db.create_mistake(&mistake).await?;
            created.id
        }
        "style" => {
            let rule = crate::models::StyleRule {
                id: None,
                description: content.clone(),
                example: String::new(),
            };
            let created = db.create_style_rule(&rule).await?;
            created.id
        }
        "security" => {
            let detail = crate::models::SecurityDetail {
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
    let linked = if let (Some(target_id), Some(record_id)) = (link_to, &id) {
        db.link_context(&target_id, record_id).await?;
        true
    } else {
        false
    };

    Ok(KnowledgeResult {
        kind,
        content,
        linked,
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
            None,
            &db,
        ).await.expect("add_knowledge should succeed");
        
        assert_eq!(result.kind, "mistake");
        assert_eq!(result.content, "Using unwrap");
        assert!(!result.linked);
        
        let mistakes = db.list_mistakes().await.expect("list_mistakes should succeed");
        assert_eq!(mistakes.len(), 1);
        assert_eq!(mistakes[0].content, "Using unwrap");
    }

    #[tokio::test]
    async fn test_add_style_rule() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");
        
        let result = crate::ingest::add_knowledge(
            "style".to_string(),
            "Use match instead of unwrap".to_string(),
            None,
            &db,
        ).await.expect("add_knowledge should succeed");
        
        assert_eq!(result.kind, "style");
        assert!(!result.linked);
        
        let rules = db.list_style_rules().await.expect("list_style_rules should succeed");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].description, "Use match instead of unwrap");
    }

    #[tokio::test]
    async fn test_add_security_detail() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");
        
        let result = crate::ingest::add_knowledge(
            "security".to_string(),
            "SQL injection risk".to_string(),
            None,
            &db,
        ).await.expect("add_knowledge should succeed");
        
        assert_eq!(result.kind, "security");
        assert!(!result.linked);
        
        let details = db.list_security_details().await.expect("list_security_details should succeed");
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].content, "SQL injection risk");
    }

    #[tokio::test]
    async fn test_add_knowledge_invalid_kind() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");
        
        let result = crate::ingest::add_knowledge(
            "invalid".to_string(),
            "Some content".to_string(),
            None,
            &db,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown knowledge type"));
    }

    #[tokio::test]
    async fn test_add_knowledge_with_link() {
        let db = crate::db::DB::new("mem://").await.expect("Failed to init DB");
        
        let project = db.create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "Test".to_string(),
        }).await.expect("create project");
        let project_id = project.id.expect("project id");
        
        let result = crate::ingest::add_knowledge(
            "mistake".to_string(),
            "Using unwrap".to_string(),
            Some(project_id.clone()),
            &db,
        ).await.expect("add_knowledge should succeed");
        
        assert!(result.linked);
    }
}
