use crate::db::surreal::DB;
use crate::db::surreal::util::{ensure_record_id, json_to_surreal, surreal_to_json};

impl DB {
    /// Internal helper: creates a module record without any relationships.
    pub(crate) async fn create_module_record(
        &self,
        module: &crate::models::Module,
    ) -> anyhow::Result<crate::models::Module> {
        let json = serde_json::to_value(module)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("module").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create module"))?;
        let result: crate::models::Module = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a module and RELATEs it to its parent project and optionally a parent module.
    ///
    /// - If `parent_module_id` is provided: `parent_module contains child_module` +
    ///   `child_module belongs_to_module parent_module`
    /// - Top-level only (no parent module): `project contains module`
    /// - Always: `module belongs_to_project project`
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        notes: Option<&str>,
        project_id: &str,
        parent_module_id: Option<&str>,
    ) -> anyhow::Result<crate::models::Module> {
        let module = crate::models::Module {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_module_record(&module).await?;
        if let Some(mid) = result.id.as_ref() {
            ensure_record_id("project", project_id)?;
            self.link(mid, "belongs_to_project", project_id).await?;
            if let Some(parent_mid) = parent_module_id {
                ensure_record_id("module", parent_mid)?;
                self.link(parent_mid, "has_module", mid).await?;
                self.link(mid, "belongs_to_module", parent_mid).await?;
            } else {
                self.link(project_id, "contains", mid).await?;
            }
        }
        Ok(result)
    }

    /// Fetches a module by record id.
    pub async fn get_module(&self, id: &str) -> anyhow::Result<Option<crate::models::Module>> {
        self.get_record("module", id).await
    }

    /// Lists modules directly under a project via graph traversal (top-level modules only).
    pub async fn list_modules_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Module>> {
        ensure_record_id("project", project_id)?;
        self.query_graph_list(
            "SELECT ->contains->module.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists child modules directly under a given module via graph traversal.
    pub async fn list_modules_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Module>> {
        ensure_record_id("module", module_id)?;
        self.query_graph_list(
            "SELECT ->has_module->module.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all modules (unfiltered).
    pub async fn list_modules(&self) -> anyhow::Result<Vec<crate::models::Module>> {
        self.list_records("module").await
    }

    /// Gets context for a module (Module node only).
    pub async fn get_module_context_node(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.get_linked_context(module_id).await
    }

    /// Gets full context for a module, walking up all ancestor modules and the project.
    pub async fn get_module_context_full(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        let mut contexts = self.get_module_context_node(module_id).await?;
        let h = self.resolve_structural_ancestry(module_id).await?;

        // Walk up ancestor modules (excluding self)
        for mid in &h.module_ids {
            if mid != module_id {
                contexts.extend(self.get_linked_context(mid).await?);
            }
        }
        for pid in &h.project_ids {
            contexts.extend(self.get_linked_context(pid).await?);
        }

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        contexts.retain(|c| {
            if let Some(cid) = &c.id {
                seen.insert(cid.clone())
            } else {
                true
            }
        });
        Ok(contexts)
    }

    /// Gets context for a module.
    pub async fn get_module_context(
        &self,
        module_id: &str,
        full: bool,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        if full {
            self.get_module_context_full(module_id).await
        } else {
            self.get_module_context_node(module_id).await
        }
    }

    /// Updates a module's name, description, or notes.
    pub async fn update_module(
        &self,
        module_id: &str,
        name: Option<String>,
        description: Option<String>,
        notes: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Module>> {
        let key = module_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(module_id);

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
        if let Some(notes) = notes {
            patch.insert("notes".to_string(), serde_json::Value::String(notes));
        }

        if patch.is_empty() {
            return self.get_module(module_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("module", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes a module by id.
    pub async fn delete_module(&self, module_id: &str) -> anyhow::Result<bool> {
        let key = module_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(module_id);

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("module", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_id_must_have_project_prefix_for_module_ops() {
        ensure_record_id("project", "project:abc")
            .expect("list_modules_by_project accepts project:id");
        let err = ensure_record_id("project", "module:1").expect_err("wrong table rejected");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[test]
    fn module_id_must_have_module_prefix_for_child_module_ops() {
        ensure_record_id("module", "module:xyz")
            .expect("list_modules_by_module accepts module:id");
        let err = ensure_record_id("module", "project:1").expect_err("wrong table rejected");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[tokio::test]
    async fn test_delete_module_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();
        let module = db
            .create_module("DeleteModule", "test", None, &project_id, None)
            .await
            .expect("create");
        let id = module.id.unwrap();

        let deleted = db.delete_module(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_module(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_create_child_module_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();
        let parent = db
            .create_module("Parent", "parent module", None, &project_id, None)
            .await
            .expect("create parent");
        let parent_id = parent.id.unwrap();

        let child = db
            .create_module("Child", "child module", None, &project_id, Some(&parent_id))
            .await
            .expect("create child");
        let child_id = child.id.unwrap();

        let children = db.list_modules_by_module(&parent_id).await.expect("list");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id.as_deref(), Some(child_id.as_str()));
    }
}
