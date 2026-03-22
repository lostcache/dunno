use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

/// Validates workflow creation parameters.
pub(crate) fn validate_workflow_params(name: &str, content: &str) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!("Workflow name cannot be empty"));
    }
    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("Workflow content cannot be empty"));
    }
    if name.len() > 255 {
        return Err(anyhow::anyhow!("Workflow name too long (max 255 chars)"));
    }
    Ok(())
}

impl DB {
    /// Internal helper: creates a workflow record without any relationships.
    pub(crate) async fn create_workflow_record(
        &self,
        workflow: &crate::models::Workflow,
    ) -> anyhow::Result<crate::models::Workflow> {
        let json = serde_json::to_value(workflow)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("workflow").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create workflow"))?;
        let result: crate::models::Workflow = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a workflow and RELATEs it to its parent project with bidirectional edges.
    pub async fn create_workflow(
        &self,
        name: &str,
        content: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Workflow> {
        validate_workflow_params(name, content)?;

        let workflow = crate::models::Workflow {
            id: None,
            name: name.to_string(),
            content: content.to_string(),
        };
        let result = self.create_workflow_record(&workflow).await?;

        if let Some(wid) = result.id.as_ref() {
            self.link(project_id, "has_workflow", wid).await?;
            self.link(wid, "belongs_to_project", project_id).await?;
        }

        Ok(result)
    }

    /// Fetches a workflow by record id.
    pub async fn get_workflow(&self, id: &str) -> anyhow::Result<Option<crate::models::Workflow>> {
        self.get_record("workflow", id).await
    }

    /// Returns all workflows (unfiltered).
    pub async fn list_workflows(&self) -> anyhow::Result<Vec<crate::models::Workflow>> {
        self.list_records("workflow").await
    }

    /// Lists workflows under a project via graph traversal.
    pub async fn list_workflows_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Workflow>> {
        self.query_graph_list(
            "SELECT ->has_workflow->workflow.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Updates a workflow's name or content.
    pub async fn update_workflow(
        &self,
        workflow_id: &str,
        name: Option<String>,
        content: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Workflow>> {
        let key = workflow_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(workflow_id);

        let mut patch = serde_json::Map::new();
        if let Some(name) = name {
            patch.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(content) = content {
            patch.insert("content".to_string(), serde_json::Value::String(content));
        }

        if patch.is_empty() {
            return self.get_workflow(workflow_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("workflow", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes a workflow by id.
    pub async fn delete_workflow(&self, workflow_id: &str) -> anyhow::Result<bool> {
        let key = workflow_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(workflow_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("workflow", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_workflow_params_accepts_valid_input() {
        validate_workflow_params("Valid Name", "Valid content")
            .expect("should accept valid params");
    }

    #[test]
    fn validate_workflow_params_rejects_empty_name() {
        let err = validate_workflow_params("", "content").expect_err("empty name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_workflow_params_rejects_whitespace_only_name() {
        let err =
            validate_workflow_params("   ", "content").expect_err("whitespace name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_workflow_params_rejects_empty_content() {
        let err = validate_workflow_params("Name", "").expect_err("empty content should fail");
        assert!(err.to_string().contains("content"));
    }

    #[test]
    fn validate_workflow_params_rejects_long_name() {
        let long_name = "a".repeat(256);
        let err =
            validate_workflow_params(&long_name, "content").expect_err("long name should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[tokio::test]
    async fn test_delete_workflow_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let workflow = db
            .create_workflow("DeleteWorkflow", "test content", "project:1")
            .await
            .expect("create");
        let id = workflow.id.unwrap();

        let deleted = db.delete_workflow(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_workflow(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
