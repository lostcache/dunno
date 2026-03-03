use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

fn ensure_project_record_id(id: &str) -> anyhow::Result<()> {
    if id.starts_with("project:") {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Expected project record id (project:...), got {:?}",
            id
        ))
    }
}

impl DB {
    /// Internal helper: creates a new project record without any relationships.
    pub(crate) async fn create_project_record(
        &self,
        project: &crate::models::Project,
    ) -> anyhow::Result<crate::models::Project> {
        let json = serde_json::to_value(project)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("project").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create project"))?;
        let json = surreal_to_json(val);
        Ok(serde_json::from_value(json)?)
    }

    /// Creates a new project record (no relationships).
    pub async fn create_project(
        &self,
        project: &crate::models::Project,
    ) -> anyhow::Result<crate::models::Project> {
        self.create_project_record(project).await
    }

    /// Fetches a project by record id.
    pub async fn get_project(&self, id: &str) -> anyhow::Result<Option<crate::models::Project>> {
        self.get_record("project", id).await
    }

    /// Returns all projects.
    pub async fn list_projects(&self) -> anyhow::Result<Vec<crate::models::Project>> {
        self.list_records("project").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_project_record_id_accepts_project_prefix() {
        ensure_project_record_id("project:abc").expect("should accept project record id");
        ensure_project_record_id("project:123").expect("should accept project record id");
    }

    #[test]
    fn ensure_project_record_id_rejects_wrong_prefix_or_bare_id() {
        let err = ensure_project_record_id("module:1").expect_err("wrong table should fail");
        assert!(err.to_string().contains("Expected project record id"));

        let err = ensure_project_record_id("abc").expect_err("bare id should fail");
        assert!(err.to_string().contains("Expected project record id"));
    }
}
