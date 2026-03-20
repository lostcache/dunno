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

    /// Creates a module and RELATEs it to its parent project.
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        notes: Option<&str>,
        project_id: &str,
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
            self.link(project_id, "contains", mid).await?;
            self.link(mid, "belongs_to_project", project_id).await?;
        }
        Ok(result)
    }

    /// Fetches a module by record id.
    pub async fn get_module(&self, id: &str) -> anyhow::Result<Option<crate::models::Module>> {
        self.get_record("module", id).await
    }

    /// Lists modules under a project via graph traversal.
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

    /// Returns all modules (unfiltered).
    pub async fn list_modules(&self) -> anyhow::Result<Vec<crate::models::Module>> {
        self.list_records("module").await
    }

    /// Internal helper: creates a submodule record without any relationships.
    pub(crate) async fn create_submodule_record(
        &self,
        submodule: &crate::models::Submodule,
    ) -> anyhow::Result<crate::models::Submodule> {
        let json = serde_json::to_value(submodule)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("submodule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create submodule"))?;
        let result: crate::models::Submodule = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a submodule and RELATEs it to its parent module and project.
    pub async fn create_submodule(
        &self,
        name: &str,
        description: &str,
        notes: Option<&str>,
        module_id: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Submodule> {
        let submodule = crate::models::Submodule {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_submodule_record(&submodule).await?;
        if let Some(sub_id) = result.id.as_ref() {
            ensure_record_id("module", module_id)?;
            self.link(module_id, "contains", sub_id).await?;
            self.link(sub_id, "belongs_to_module", module_id).await?;
            self.link(sub_id, "belongs_to_project", project_id).await?;
        }
        Ok(result)
    }

    /// Fetches a submodule by record id.
    pub async fn get_submodule(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Submodule>> {
        self.get_record("submodule", id).await
    }

    /// Returns all submodules.
    pub async fn list_submodules(&self) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.list_records("submodule").await
    }

    /// Lists submodules under a module via graph traversal.
    pub async fn list_submodules_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Submodule>> {
        ensure_record_id("module", module_id)?;
        self.query_graph_list(
            "SELECT ->contains->submodule.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists submodules under a project via belongs_to_project edge.
    pub async fn list_submodules_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Submodule>> {
        ensure_record_id("project", project_id)?;
        self.query_graph_list(
            "SELECT <-belongs_to_project<-submodule.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Gets context for a module (Module node only).
    pub async fn get_module_context_node(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.get_linked_context(module_id).await
    }

    /// Gets full context for a module (Module + Project).
    pub async fn get_module_context_full(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        let mut contexts = self.get_module_context_node(module_id).await?;
        let h = self.resolve_structural_ancestry(module_id).await?;
        for pid in h.project_ids {
            if pid != module_id {
                contexts.extend(self.get_linked_context(&pid).await?);
            }
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

    /// Gets context for a submodule (Submodule node only).
    pub async fn get_submodule_context_node(
        &self,
        submodule_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.get_linked_context(submodule_id).await
    }

    /// Gets full context for a submodule (Submodule + Module + Project).
    pub async fn get_submodule_context_full(
        &self,
        submodule_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        let mut contexts = self.get_submodule_context_node(submodule_id).await?;
        let h = self.resolve_structural_ancestry(submodule_id).await?;

        for mid in h.module_ids {
            if mid != submodule_id {
                contexts.extend(self.get_linked_context(&mid).await?);
            }
        }
        for pid in h.project_ids {
            if pid != submodule_id {
                contexts.extend(self.get_linked_context(&pid).await?);
            }
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

    /// Gets context for a submodule.
    pub async fn get_submodule_context(
        &self,
        submodule_id: &str,
        full: bool,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        if full {
            self.get_submodule_context_full(submodule_id).await
        } else {
            self.get_submodule_context_node(submodule_id).await
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

    /// Updates a submodule's name, description, or notes.
    pub async fn update_submodule(
        &self,
        submodule_id: &str,
        name: Option<String>,
        description: Option<String>,
        notes: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Submodule>> {
        let key = submodule_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(submodule_id);

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
            return self.get_submodule(submodule_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("submodule", key))
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

    /// Deletes a submodule by id.
    pub async fn delete_submodule(&self, submodule_id: &str) -> anyhow::Result<bool> {
        let key = submodule_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(submodule_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("submodule", key)).await?;
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
    fn module_id_must_have_module_prefix_for_submodule_ops() {
        ensure_record_id("module", "module:xyz")
            .expect("list_submodules_by_module accepts module:id");
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
            .create_module("DeleteModule", "test", None, &project_id)
            .await
            .expect("create");
        let id = module.id.unwrap();

        let deleted = db.delete_module(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_module(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_submodule_success() {
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
            .create_module("M", "d", None, &project_id)
            .await
            .expect("create module");
        let module_id = module.id.unwrap();
        let submodule = db
            .create_submodule("DeleteSubmodule", "test", None, &module_id, &project_id)
            .await
            .expect("create");
        let id = submodule.id.unwrap();

        let deleted = db.delete_submodule(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_submodule(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
