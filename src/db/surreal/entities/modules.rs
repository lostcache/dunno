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

    /// Creates a module and optionally RELATEs it to its parent project.
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        notes: Option<&str>,
        project_id: Option<&str>,
    ) -> anyhow::Result<crate::models::Module> {
        let module = crate::models::Module {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_module_record(&module).await?;
        if let (Some(pid), Some(mid)) = (project_id, result.id.as_ref()) {
            ensure_record_id("project", pid)?;
            self.link(pid, "contains", mid).await?;
            // Add bidirectional edge: module -> belongs_to_project -> project
            self.link(mid, "belongs_to_project", pid).await?;
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

    /// Creates a submodule and optionally RELATEs it to its parent module.
    pub async fn create_submodule(
        &self,
        name: &str,
        description: &str,
        notes: Option<&str>,
        module_id: Option<&str>,
    ) -> anyhow::Result<crate::models::Submodule> {
        let submodule = crate::models::Submodule {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_submodule_record(&submodule).await?;
        if let (Some(mid), Some(sub_id)) = (module_id, result.id.as_ref()) {
            ensure_record_id("module", mid)?;
            self.link(mid, "contains", sub_id).await?;
            // Add bidirectional edge: submodule -> belongs_to_module -> module
            self.link(sub_id, "belongs_to_module", mid).await?;
            // Get module's project and link submodule to project
            let mut response = self
                .client
                .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($mid)")
                .bind(("mid", mid.to_string()))
                .await?;
            let project_record: Option<surrealdb::types::Value> = response.take(0)?;
            if let Some(record) = project_record {
                let json = surreal_to_json(record);
                if let Some(project_id) = json
                    .get("pid")
                    .and_then(|p| p.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                {
                    self.link(sub_id, "belongs_to_project", project_id).await?;
                }
            }
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
}
