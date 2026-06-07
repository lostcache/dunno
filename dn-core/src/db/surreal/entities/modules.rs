use crate::db::surreal::DB;
use crate::db::surreal::util::{ensure_record_id, json_to_surreal, surreal_to_json};

impl DB {
    /// Creates a module and RELATEs it to its parent project and optionally a parent module.
    /// All operations run inside a transaction; if any step fails, the transaction is cancelled.
    ///
    /// - If `parent_module_id` is provided: `parent_module contains child_module` +
    ///   `child_module belongs_to_module parent_module`
    /// - Top-level only (no parent module): `project contains module`
    /// - Always: `module belongs_to_project project`
    pub async fn create_modules(
        &self,
        names: Vec<String>,
        descriptions: Vec<String>,
        project_id: &str,
        parent_module_ids: Vec<String>,
    ) -> anyhow::Result<Vec<crate::models::Module>> {
        let tx = self.client.clone().begin().await?;

        let mut results: Vec<crate::models::Module> = vec![];

        for ((n, desc), paren) in
            std::iter::zip(std::iter::zip(names, descriptions), parent_module_ids)
        {
            let res = async {
                let maybe_paren_module = if paren.len() > 0 { Some(paren) } else { None };
                let module = crate::models::Module {
                    id: None,
                    name: n.to_string(),
                    description: desc.to_string(),
                    parent_module_id: maybe_paren_module,
                };
                let json = serde_json::to_value(&module)?;
                let value = json_to_surreal(json);
                let created = tx.create("module").content(value).await?;
                match created {
                    Some(val) => {
                        let created_module: crate::models::Module =
                            serde_json::from_value(surreal_to_json(val))?;
                        if let Some(mid) = created_module.id.as_ref() {
                            // Always create belongs_to_project edge
                            let from_rid = surrealdb::types::RecordId::parse_simple(mid)
                                .map_err(|_| anyhow::anyhow!("Invalid record id: {}", mid))?;
                            let to_rid = surrealdb::types::RecordId::parse_simple(project_id)
                                .map_err(|_| {
                                    anyhow::anyhow!("Invalid record id: {}", project_id)
                                })?;
                            tx.query("RELATE $from->belongs_to_project->$to")
                                .bind(("from", from_rid))
                                .bind(("to", to_rid))
                                .await?;

                            if let Some(pm) = module.parent_module_id {
                                ensure_record_id("module", &pm)?;
                                let from_rid = surrealdb::types::RecordId::parse_simple(&pm)
                                    .map_err(|_| anyhow::anyhow!("Invalid record id: {}", pm))?;
                                let to_rid = surrealdb::types::RecordId::parse_simple(mid)
                                    .map_err(|_| anyhow::anyhow!("Invalid record id: {}", mid))?;
                                tx.query("RELATE $from->has_module->$to")
                                    .bind(("from", from_rid))
                                    .bind(("to", to_rid))
                                    .await?;

                                let from_rid = surrealdb::types::RecordId::parse_simple(mid)
                                    .map_err(|_| anyhow::anyhow!("Invalid record id: {}", mid))?;
                                let to_rid = surrealdb::types::RecordId::parse_simple(&pm)
                                    .map_err(|_| anyhow::anyhow!("Invalid record id: {}", pm))?;
                                tx.query("RELATE $from->belongs_to_module->$to")
                                    .bind(("from", from_rid))
                                    .bind(("to", to_rid))
                                    .await?;
                            } else {
                                let from_rid = surrealdb::types::RecordId::parse_simple(project_id)
                                    .map_err(|_| {
                                        anyhow::anyhow!("Invalid record id: {}", project_id)
                                    })?;
                                let to_rid = surrealdb::types::RecordId::parse_simple(mid)
                                    .map_err(|_| anyhow::anyhow!("Invalid record id: {}", mid))?;
                                tx.query("RELATE $from->has_module->$to")
                                    .bind(("from", from_rid))
                                    .bind(("to", to_rid))
                                    .await?;
                            }
                        }
                        Ok(created_module)
                    }
                    None => {
                        anyhow::bail!("Failed to create module/s")
                    }
                }
            };

            match res.await {
                Ok(module) => {
                    results.push(module);
                }
                Err(e) => {
                    tx.cancel().await?;
                    anyhow::bail!("Failed to create module/s: {}", e)
                }
            }
        }

        tx.commit().await?;
        Ok(results)
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
            "SELECT ->has_module->module.* AS items FROM ONLY type::record($pid)",
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
        ensure_record_id("module", "module:xyz").expect("list_modules_by_module accepts module:id");
        let err = ensure_record_id("module", "project:1").expect_err("wrong table rejected");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[tokio::test]
    async fn test_delete_module_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id;
        let module = db
            .create_modules(
                vec!["DeleteModule".to_string()],
                vec!["test".to_string()],
                &project_id,
                vec!["".to_string()],
            )
            .await
            .expect("create");
        let module = module.into_iter().next().expect("module created");
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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id;
        let parent = db
            .create_modules(
                vec!["Parent".to_string()],
                vec!["parent module".to_string()],
                &project_id,
                vec!["".to_string()],
            )
            .await
            .expect("create parent");
        let parent = parent.into_iter().next().expect("module created");
        let parent_id = parent.id.unwrap();

        let child = db
            .create_modules(
                vec!["Child".to_string()],
                vec!["child module".to_string()],
                &project_id,
                vec![parent_id.clone()],
            )
            .await
            .expect("create child");
        let child = child.into_iter().next().expect("module created");
        let child_id = child.id.unwrap();

        let children = db.list_modules_by_module(&parent_id).await.expect("list");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id.as_deref(), Some(child_id.as_str()));
    }
}
