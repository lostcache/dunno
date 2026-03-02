use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::DB;

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
    pub async fn get_project(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Project>> {
        self.get_record("project", id).await
    }

    /// Returns all projects.
    pub async fn list_projects(&self) -> anyhow::Result<Vec<crate::models::Project>> {
        self.list_records("project").await
    }
}
