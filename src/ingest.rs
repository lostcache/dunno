#[derive(Debug)]
pub struct KnowledgeResult {
    pub kind: String,
    pub content: String,
    pub linked: bool,
}

pub async fn add_knowledge_schemaless(
    fields: serde_json::Map<String, serde_json::Value>,
    link_to: Vec<String>,
    db: &crate::db::DB,
) -> anyhow::Result<KnowledgeResult> {
    if fields.is_empty() {
        return Err(anyhow::anyhow!(
            "At least one --field key=value pair is required"
        ));
    }

    let kind = fields
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let content = fields
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let created = db.create_context_schemaless(fields).await?;
    let record_id = match created.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
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
    use serde_json::json;

    // Schemaless mode tests
    #[tokio::test]
    async fn test_add_knowledge_schemaless_basic() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("performance".to_string()),
        );
        fields.insert(
            "content".to_string(),
            serde_json::Value::String("Use parallel iterators".to_string()),
        );
        fields.insert(
            "category".to_string(),
            serde_json::Value::String("optimization".to_string()),
        );

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        assert_eq!(result.kind, "performance");
        assert_eq!(result.content, "Use parallel iterators");
        assert!(!result.linked);

        // Verify the record was created with custom fields
        let contexts = db
            .list_contexts()
            .await
            .expect("list_contexts should succeed");
        assert_eq!(contexts.len(), 1);
        // Note: The legacy Context struct won't have the custom 'category' field,
        // but the data is stored in the database
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_with_link() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.expect("project id");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("security".to_string()),
        );
        fields.insert(
            "content".to_string(),
            serde_json::Value::String("Validate inputs".to_string()),
        );
        fields.insert(
            "severity".to_string(),
            serde_json::Value::String("high".to_string()),
        );

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![project_id.clone()], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        assert!(result.linked);
        assert_eq!(result.kind, "security");
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_empty_fields() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let fields = serde_json::Map::new();

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("At least one --field")
        );
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_defaults() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        // Only content, no type
        fields.insert(
            "content".to_string(),
            serde_json::Value::String("Some content".to_string()),
        );

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        // Should default to "unknown" type
        assert_eq!(result.kind, "unknown");
        assert_eq!(result.content, "Some content");
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_multiple_links() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module(
                &"Auth".to_string(),
                &"Auth module".to_string(),
                None,
                &project_id,
            )
            .await
            .expect("create module");
        let module_id = module.id.expect("module id");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("deployment".to_string()),
        );
        fields.insert(
            "content".to_string(),
            serde_json::Value::String("Backup DB".to_string()),
        );

        let result = crate::ingest::add_knowledge_schemaless(
            fields,
            vec![project_id.clone(), module_id.clone()],
            &db,
        )
        .await
        .expect("add_knowledge_schemaless should succeed");

        assert!(result.linked);
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_complex_types() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert("type".to_string(), json!("performance"));
        fields.insert("content".to_string(), json!("Use parallel iterators"));
        fields.insert("priority".to_string(), json!(5));
        fields.insert("tags".to_string(), json!(["optimization", "rust"]));
        fields.insert(
            "metadata".to_string(),
            json!({"author": "test", "version": 1}),
        );

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        assert_eq!(result.kind, "performance");
        assert_eq!(result.content, "Use parallel iterators");
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_no_content() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        // Only type, no content
        fields.insert("type".to_string(), json!("note"));
        fields.insert("importance".to_string(), json!("high"));

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        assert_eq!(result.kind, "note");
        assert_eq!(result.content, ""); // Empty string default
    }

    #[tokio::test]
    async fn test_add_knowledge_schemaless_unicode_and_special_chars() {
        let db = crate::db::DB::new("mem://")
            .await
            .expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert("type".to_string(), json!("mistake"));
        fields.insert("content".to_string(), json!("日本語テキスト and emojis 🎉"));
        fields.insert(
            "description".to_string(),
            json!("Special chars: \"quoted\" and =equals="),
        );

        let result = crate::ingest::add_knowledge_schemaless(fields, vec![], &db)
            .await
            .expect("add_knowledge_schemaless should succeed");

        assert_eq!(result.kind, "mistake");
        assert_eq!(result.content, "日本語テキスト and emojis 🎉");
    }
}
