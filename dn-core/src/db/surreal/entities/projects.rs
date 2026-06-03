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
        if let Some(existing) = self.get_project_by_name(&project.name, false).await? {
            return Err(anyhow::anyhow!(
                "Project with name '{}' already exists (id: {})",
                project.name,
                existing.id,
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

    /// Gets context for a project (Project node only).
    pub async fn get_project_context_node(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.get_linked_context(project_id).await
    }

    /// Gets full context for a project (same as node since it's the root).
    pub async fn get_project_context_full(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.get_project_context_node(project_id).await
    }

    /// Gets context for a project.
    pub async fn get_project_context(
        &self,
        project_id: &str,
        full: bool,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        if full {
            self.get_project_context_full(project_id).await
        } else {
            self.get_project_context_node(project_id).await
        }
    }

    /// Updates a project's name or description.
    pub async fn update_project(
        &self,
        project_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Project>> {
        let key = project_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(project_id);

        let mut patch = serde_json::Map::new();
        if let Some(name) = name {
            patch.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(description) = description {
            patch.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        if patch.is_empty() {
            return self.get_project(project_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("project", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Returns the structural hierarchy of a project: all modules recursively with their files.
    pub async fn get_project_structure(
        &self,
        project_id: &str,
    ) -> anyhow::Result<crate::models::ProjectStructure> {
        let project = self
            .get_project(project_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Project not found: {}", project_id))?;

        let top_modules = self.list_modules_by_project(project_id).await?;
        let mut module_structures = Vec::new();

        for module in top_modules {
            let ms = self.build_module_structure(module).await?;
            module_structures.push(ms);
        }

        Ok(crate::models::ProjectStructure {
            project,
            modules: module_structures,
        })
    }

    /// Recursively builds a ModuleStructure for a module and all its descendant modules.
    async fn build_module_structure(
        &self,
        module: crate::models::Module,
    ) -> anyhow::Result<crate::models::ModuleStructure> {
        let module_id = module
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Module has no ID"))?;

        let files = self.list_files_by_module(module_id).await?;
        let child_modules = self.list_modules_by_module(module_id).await?;

        let mut children = Vec::new();
        for child in child_modules {
            let cs = Box::pin(self.build_module_structure(child)).await?;
            children.push(cs);
        }

        Ok(crate::models::ModuleStructure {
            module,
            children,
            files,
        })
    }

    /// Deletes a project by id.
    pub async fn delete_project(&self, project_id: &str) -> anyhow::Result<bool> {
        let key = project_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(project_id);

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("project", key)).await?;
        Ok(deleted.is_some())
    }
}

pub async fn get_project_structure_json(
    project_id: &str,
    db: &DB,
) -> anyhow::Result<crate::models::ProjectStructure> {
    db.get_project_structure(project_id).await
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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "UniqueProject".to_string(),
            description: "First".to_string(),
        })
        .await
        .expect("First project should succeed");

        // Try to create second project with same name
        let result = db
            .create_project(&crate::models::Project {
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
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

    #[tokio::test]
    async fn test_delete_project_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "DeleteMe".to_string(),
                description: "test".to_string(),
            })
            .await
            .expect("create");
        let id = project.id;

        let deleted = db.delete_project(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_project(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
