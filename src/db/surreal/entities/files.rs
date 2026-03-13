use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

fn ensure_one_of_record_ids<'a>(tables: &[&'a str], id: &'a str) -> anyhow::Result<&'a str> {
    for table in tables {
        if id.starts_with(&format!("{table}:")) {
            return Ok(*table);
        }
    }
    Err(anyhow::anyhow!(
        "Expected record id for one of {:?}, got {:?}",
        tables,
        id
    ))
}

impl DB {
    /// Internal helper: creates a file record without any relationships.
    pub(crate) async fn create_file_record(
        &self,
        file: &crate::models::File,
    ) -> anyhow::Result<crate::models::File> {
        let json = serde_json::to_value(file)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("file").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create file"))?;
        let result: crate::models::File = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a file and optionally RELATEs it to a parent (module or submodule).
    pub async fn create_file(
        &self,
        name: &str,
        path: &str,
        description: Option<&str>,
        notes: Option<&str>,
        parent_id: Option<&str>,
    ) -> anyhow::Result<crate::models::File> {
        let file = crate::models::File {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
            description: description.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_file_record(&file).await?;
        if let (Some(pid), Some(fid)) = (parent_id, result.id.as_ref()) {
            let table = ensure_one_of_record_ids(&["module", "submodule"], pid)?;
            self.link(pid, "contains", fid).await?;
            // Add bidirectional edges based on parent type
            if table == "module" {
                // File -> belongs_to_module -> module
                self.link(fid, "belongs_to_module", pid).await?;
                // Get module's project and link file to project
                let mut response = self
                    .client
                    .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($mid)")
                    .bind(("mid", pid.to_string()))
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
                        self.link(fid, "belongs_to_project", project_id).await?;
                    }
                }
            } else if table == "submodule" {
                // File -> belongs_to_submodule -> submodule
                self.link(fid, "belongs_to_submodule", pid).await?;
                // Get submodule's module and project
                let mut response = self
                    .client
                    .query("SELECT ->belongs_to_module->module.id AS mid, ->belongs_to_project->project.id AS pid FROM ONLY type::record($sid)")
                    .bind(("sid", pid.to_string()))
                    .await?;
                let submodule_record: Option<surrealdb::types::Value> = response.take(0)?;
                if let Some(record) = submodule_record {
                    let json = surreal_to_json(record);
                    // Link to module
                    if let Some(module_id) = json
                        .get("mid")
                        .and_then(|m| m.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                    {
                        self.link(fid, "belongs_to_module", module_id).await?;
                    }
                    // Link to project
                    if let Some(project_id) = json
                        .get("pid")
                        .and_then(|p| p.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                    {
                        self.link(fid, "belongs_to_project", project_id).await?;
                    }
                }
            }
        }
        Ok(result)
    }

    /// Fetches a file by record id.
    pub async fn get_file(&self, id: &str) -> anyhow::Result<Option<crate::models::File>> {
        self.get_record("file", id).await
    }

    /// Returns all files.
    pub async fn list_files(&self) -> anyhow::Result<Vec<crate::models::File>> {
        self.list_records("file").await
    }

    /// Lists files under a module via graph traversal.
    pub async fn list_files_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        let _ = ensure_one_of_record_ids(&["module"], module_id)?;
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files under a submodule via graph traversal.
    pub async fn list_files_by_submodule(
        &self,
        submodule_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        let _ = ensure_one_of_record_ids(&["submodule"], submodule_id)?;
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($sid)",
            "sid",
            submodule_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files under a project via belongs_to_project edge.
    pub async fn list_files_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        ensure_one_of_record_ids(&["project"], project_id)?;
        self.query_graph_list(
            "SELECT <-belongs_to_project<-file.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Gets context only for the specific file node.
    pub async fn get_file_context_node(
        &self,
        file_id: &str,
    ) -> anyhow::Result<crate::models::FileContext> {
        let file = self
            .get_file(file_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", file_id))?;
        let contexts = self.get_linked_context(file_id).await?;
        Ok(crate::models::FileContext { file, contexts })
    }

    /// Gets full inherited context for a file (Project -> Module -> Submodule -> File).
    pub async fn get_file_context_full(
        &self,
        file_id: &str,
    ) -> anyhow::Result<crate::models::FileContext> {
        let mut ctx = self.get_file_context_node(file_id).await?;

        // Resolve ancestry for this file
        let ancestry = self.resolve_structural_ancestry(file_id).await?;

        for sid in ancestry.submodule_ids {
            ctx.contexts.extend(self.get_linked_context(&sid).await?);
        }
        for mid in ancestry.module_ids {
            ctx.contexts.extend(self.get_linked_context(&mid).await?);
        }
        for pid in ancestry.project_ids {
            ctx.contexts.extend(self.get_linked_context(&pid).await?);
        }

        // Deduplicate contexts by ID
        let mut seen = std::collections::HashSet::new();
        ctx.contexts.retain(|c| {
            if let Some(id) = &c.id {
                seen.insert(id.clone())
            } else {
                true
            }
        });

        Ok(ctx)
    }

    /// Gets context for a file, optionally including parent hierarchy.
    pub async fn get_file_context(
        &self,
        file_id: &str,
        full: bool,
    ) -> anyhow::Result<crate::models::FileContext> {
        if full {
            self.get_file_context_full(file_id).await
        } else {
            self.get_file_context_node(file_id).await
        }
    }

    /// Deletes a file by id.
    pub async fn delete_file(&self, file_id: &str) -> anyhow::Result<bool> {
        let key = file_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(file_id);

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("file", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_one_of_record_ids_accepts_expected_tables() {
        let table = ensure_one_of_record_ids(&["module", "submodule"], "module:1")
            .expect("should accept module record id");
        assert_eq!(table, "module");

        let table = ensure_one_of_record_ids(&["module", "submodule"], "submodule:1")
            .expect("should accept submodule record id");
        assert_eq!(table, "submodule");
    }

    #[test]
    fn ensure_one_of_record_ids_rejects_other_tables_or_missing_prefix() {
        let err = ensure_one_of_record_ids(&["module", "submodule"], "project:1")
            .expect_err("should reject project record id");
        assert!(err.to_string().contains("Expected record id"));

        let err =
            ensure_one_of_record_ids(&["module"], "1").expect_err("should reject missing prefix");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[tokio::test]
    async fn test_delete_file_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let file = db
            .create_file("delete_me.rs", "path", None, None, None)
            .await
            .expect("create");
        let id = file.id.unwrap();

        let deleted = db.delete_file(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_file(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}

/// Returns full file context including file details and linked knowledge.
pub async fn get_file_context_json(
    file_id: &str,
    full: bool,
    db: &crate::db::DB,
) -> anyhow::Result<crate::models::FileContext> {
    db.get_file_context(file_id, full).await
}
