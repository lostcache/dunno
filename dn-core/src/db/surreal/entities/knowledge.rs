use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

fn ensure_context_record_id(to_context_id: &str) -> anyhow::Result<()> {
    if to_context_id.starts_with("context:") {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "link_context: to_context_id must be a context record id; got {:?}",
            to_context_id
        ))
    }
}

impl DB {
    /// Creates a new context record.
    pub async fn create_context(
        &self,
        context: &crate::models::Context,
    ) -> anyhow::Result<crate::models::Context> {
        let json = serde_json::to_value(context)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("context").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create context"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Creates a new context record from arbitrary key-value fields (schemaless).
    /// Returns the created record as a JSON value.
    pub async fn create_context_schemaless(
        &self,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        let json = serde_json::Value::Object(fields);
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("context").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create context"))?;
        Ok(surreal_to_json(val))
    }

    /// Fetches a context record by id.
    pub async fn get_context(&self, id: &str) -> anyhow::Result<Option<crate::models::Context>> {
        self.get_record("context", id).await
    }

    /// Returns all context records.
    pub async fn list_contexts(&self) -> anyhow::Result<Vec<crate::models::Context>> {
        self.list_records("context").await
    }

    /// Returns context records filtered by type.
    pub async fn list_contexts_by_type(
        &self,
        context_type: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        let all = self.list_contexts().await?;
        Ok(all
            .into_iter()
            .filter(|c| c.context_type == context_type)
            .collect())
    }

    /// Updates a context record with arbitrary fields (schemaless merge).
    pub async fn update_context(
        &self,
        context_id: &str,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        let key = context_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(context_id);

        if fields.is_empty() {
            let val: Option<surrealdb::types::Value> = self.client.select(("context", key)).await?;
            return Ok(val.map(surreal_to_json).unwrap_or(serde_json::Value::Null));
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("context", key))
            .merge(json_to_surreal(serde_json::Value::Object(fields)))
            .await?;

        Ok(updated
            .map(surreal_to_json)
            .unwrap_or(serde_json::Value::Null))
    }

    /// Links a structural node to a context record via has_context and creates reverse belongs_to_* edges.
    pub async fn link_context(&self, from_id: &str, to_context_id: &str) -> anyhow::Result<()> {
        ensure_context_record_id(to_context_id)?;
        self.link(from_id, "has_context", to_context_id).await?;

        let ancestry = self.resolve_structural_ancestry(from_id).await?;
        for id in ancestry.project_ids {
            self.link(to_context_id, "belongs_to_project", &id).await?;
        }
        for id in ancestry.module_ids {
            self.link(to_context_id, "belongs_to_module", &id).await?;
        }
        for id in ancestry.task_ids {
            self.link(to_context_id, "belongs_to_task", &id).await?;
        }
        for id in ancestry.epic_ids {
            self.link(to_context_id, "belongs_to_epic", &id).await?;
        }
        Ok(())
    }

    /// Deletes a context record by id.
    pub async fn delete_context(&self, context_id: &str) -> anyhow::Result<bool> {
        let key = context_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(context_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("context", key)).await?;
        Ok(deleted.is_some())
    }

    /// Returns structural node ids (project, module, task) that this context record belongs to.
    pub async fn get_belongs_to_targets(&self, context_id: &str) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        for (edge, table) in [
            ("belongs_to_project", "project"),
            ("belongs_to_module", "module"),
            ("belongs_to_task", "task"),
            ("belongs_to_epic", "epic"),
        ] {
            let ids = self
                .record_ids_from_query(
                    &format!("SELECT ->{edge}->{table}.* AS out FROM ONLY type::record($kid)"),
                    "kid",
                    context_id.to_string(),
                    "out",
                )
                .await?;
            out.extend(ids);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delete_context_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("Failed to create context");
        let context_id = created
            .get("id")
            .and_then(|v| v.as_str())
            .expect("context id")
            .to_string();

        let fetched = db
            .get_context(&context_id)
            .await
            .expect("Failed to fetch context");
        assert!(fetched.is_some());

        let deleted = db
            .delete_context(&context_id)
            .await
            .expect("Failed to delete context");
        assert!(deleted, "delete_context should return true for existing context");

        let after_delete = db
            .get_context(&context_id)
            .await
            .expect("Failed to check context");
        assert!(after_delete.is_none(), "Context should be deleted");
    }

    #[tokio::test]
    async fn delete_nonexistent_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        // Create one context to ensure the table exists
        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("seed".to_string()),
        );
        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("create context");
        let context_id = created
            .get("id")
            .and_then(|v| v.as_str())
            .expect("context id")
            .to_string();

        let deleted = db
            .delete_context("context:nonexistent12345")
            .await
            .expect("Should not error on nonexistent context when table exists");
        assert!(
            !deleted,
            "delete_context should return false for nonexistent context"
        );

        // Cleanup
        db.delete_context(&context_id).await.expect("cleanup delete");
    }

    #[test]
    fn ensure_context_record_id_accepts_context_ids() {
        ensure_context_record_id("context:1").expect("should accept context record id");
    }

    #[test]
    fn ensure_context_record_id_rejects_non_context_ids() {
        let err = ensure_context_record_id("task:1").expect_err("should reject non-context id");
        assert!(
            err.to_string()
                .contains("to_context_id must be a context record id")
        );
    }

    // Schemaless context creation tests
    #[tokio::test]
    async fn create_context_schemaless_stores_arbitrary_fields() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "custom_type".to_string(),
            serde_json::Value::String("my_type".to_string()),
        );
        fields.insert(
            "description".to_string(),
            serde_json::Value::String("Custom description".to_string()),
        );
        fields.insert("priority".to_string(), serde_json::Value::Number(5.into()));
        fields.insert(
            "tags".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("tag1".to_string()),
                serde_json::Value::String("tag2".to_string()),
            ]),
        );

        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("should create schemaless context");

        // Verify the record was created with an ID
        assert!(created.get("id").is_some());

        // Verify all custom fields were stored
        assert_eq!(
            created.get("custom_type").and_then(|v| v.as_str()),
            Some("my_type")
        );
        assert_eq!(
            created.get("description").and_then(|v| v.as_str()),
            Some("Custom description")
        );
        assert_eq!(created.get("priority").and_then(|v| v.as_i64()), Some(5));

        let tags = created.get("tags").and_then(|v| v.as_array());
        assert!(tags.is_some());
        assert_eq!(tags.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn create_context_schemaless_empty_fields() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let fields = serde_json::Map::new();

        // Should still create a record even with empty fields
        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("should create empty schemaless context");
        assert!(created.get("id").is_some());
    }

    #[tokio::test]
    async fn create_context_schemaless_with_special_characters() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "content".to_string(),
            serde_json::Value::String("Value with = equals and \"quotes\" and spaces".to_string()),
        );
        fields.insert(
            "unicode".to_string(),
            serde_json::Value::String("日本語テキスト".to_string()),
        );

        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("should create with special chars");

        assert_eq!(
            created.get("content").and_then(|v| v.as_str()),
            Some("Value with = equals and \"quotes\" and spaces")
        );
        assert_eq!(
            created.get("unicode").and_then(|v| v.as_str()),
            Some("日本語テキスト")
        );
    }

    #[tokio::test]
    async fn create_context_schemaless_returns_record_id() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_string(),
            serde_json::Value::String("test".to_string()),
        );

        let created = db
            .create_context_schemaless(fields)
            .await
            .expect("should create");

        // Verify the ID is in the correct format
        let id = created
            .get("id")
            .and_then(|v| v.as_str())
            .expect("should have id");
        assert!(id.starts_with("context:"), "id should start with context:");
    }
}
