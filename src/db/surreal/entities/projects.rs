use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

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
    /// Checks for duplicate names before creating.
    pub async fn create_project(
        &self,
        project: &crate::models::Project,
    ) -> anyhow::Result<crate::models::Project> {
        // Check for duplicate project name
        if let Some(existing) = self.get_project_by_name(&project.name, false).await? {
            return Err(anyhow::anyhow!(
                "Project with name '{}' already exists (id: {})",
                project.name,
                existing.id.as_deref().unwrap_or("unknown")
            ));
        }
        self.create_project_record(project).await
    }

    /// Fetches a project by record id.
    pub async fn get_project(&self, id: &str) -> anyhow::Result<Option<crate::models::Project>> {
        self.get_record("project", id).await
    }

    /// Fetches a project by name.
    /// When `ignore_case` is true, performs case-insensitive matching.
    pub async fn get_project_by_name(
        &self,
        name: &str,
        ignore_case: bool,
    ) -> anyhow::Result<Option<crate::models::Project>> {
        let sql = if ignore_case {
            "SELECT * FROM project WHERE string::lowercase(name) == string::lowercase($name) LIMIT 1"
        } else {
            "SELECT * FROM project WHERE name == $name LIMIT 1"
        };
        
        let mut response = self
            .client
            .query(sql)
            .bind(("name", name.to_string()))
            .await?;
        
        let fetched: Option<surrealdb::types::Value> = response.take(0)?;
        
        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Returns all projects.
    pub async fn list_projects(&self) -> anyhow::Result<Vec<crate::models::Project>> {
        self.list_records("project").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_project_by_name_case_sensitive() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        
        // Create a project with mixed case name
        let _project = db
            .create_project(&crate::models::Project {

                id: None,
                name: "MyProject".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        
        // Exact match should find it
        let found = db
            .get_project_by_name("MyProject", false)
            .await
            .expect("Failed to query");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "MyProject");
        
        // Different case should not match (case-sensitive)
        let not_found = db
            .get_project_by_name("myproject", false)
            .await
            .expect("Failed to query");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_project_by_name_case_insensitive() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        
        // Create a project with mixed case name
        let _project = db
            .create_project(&crate::models::Project {

                id: None,
                name: "MyProject".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        
        // Case-insensitive match should find it with different cases
        let found_lower = db
            .get_project_by_name("myproject", true)
            .await
            .expect("Failed to query");
        assert!(found_lower.is_some());
        assert_eq!(found_lower.unwrap().name, "MyProject");
        
        let found_upper = db
            .get_project_by_name("MYPROJECT", true)
            .await
            .expect("Failed to query");
        assert!(found_upper.is_some());
        
        let found_mixed = db
            .get_project_by_name("MyProject", true)
            .await
            .expect("Failed to query");
        assert!(found_mixed.is_some());
    }

    #[tokio::test]
    async fn test_create_project_duplicate_name_fails() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        
        // Create first project
        db.create_project(&crate::models::Project {
            id: None,
            name: "UniqueProject".to_string(),
            description: "First".to_string(),
        })
        .await
        .expect("First project should succeed");
        
        // Try to create second project with same name
        let result = db
            .create_project(&crate::models::Project {
                id: None,
                name: "UniqueProject".to_string(),
                description: "Second".to_string(),
            })
            .await;
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists"));
        assert!(err_msg.contains("UniqueProject"));
    }

    #[tokio::test]
    async fn test_get_project_by_name_not_found() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        
        let not_found = db
            .get_project_by_name("NonExistentProject", false)
            .await
            .expect("Query should not error");
        assert!(not_found.is_none());
        
        let not_found_case_insensitive = db
            .get_project_by_name("NonExistentProject", true)
            .await
            .expect("Query should not error");
        assert!(not_found_case_insensitive.is_none());
    }
}
