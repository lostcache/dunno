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

    /// Links a structural node to a context record via has_context and creates reverse belongs_to_* edges.
    pub async fn link_context(&self, from_id: &str, to_context_id: &str) -> anyhow::Result<()> {
        ensure_context_record_id(to_context_id)?;
        self.link(from_id, "has_context", to_context_id).await?;

        let hierarchy = self.resolve_structural_hierarchy(from_id).await?;
        if let Some(id) = hierarchy.project_id {
            self.link(to_context_id, "belongs_to_project", &id).await?;
        }
        if let Some(id) = hierarchy.module_id {
            self.link(to_context_id, "belongs_to_module", &id).await?;
        }
        if let Some(id) = hierarchy.task_id {
            self.link(to_context_id, "belongs_to_task", &id).await?;
        }
        Ok(())
    }

    /// Returns structural node ids (project, module, task) that this context record belongs to.
    pub async fn get_belongs_to_targets(&self, context_id: &str) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        for (edge, table) in [
            ("belongs_to_project", "project"),
            ("belongs_to_module", "module"),
            ("belongs_to_task", "task"),
        ] {
            let id = self
                .first_record_id_from_query(
                    &format!("SELECT ->{edge}->{table}.* AS out FROM ONLY type::record($kid)"),
                    "kid",
                    context_id.to_string(),
                    "out",
                )
                .await?;
            if let Some(id) = id {
                out.push(id);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
