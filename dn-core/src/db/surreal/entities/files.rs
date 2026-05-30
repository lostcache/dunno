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

    /// Creates a file, links it to a project (required), and optionally RELATEs it to a parent module.
    pub async fn create_file(
        &self,
        name: &str,
        path: &str,
        description: Option<&str>,
        notes: Option<&str>,
        project_id: &str,
        parent_id: Option<&str>,
    ) -> anyhow::Result<crate::models::File> {
        ensure_one_of_record_ids(&["project"], project_id)?;
        let file = crate::models::File {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
            description: description.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
        };
        let result = self.create_file_record(&file).await?;
        let fid = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("File created without ID"))?;
        self.link(fid, "belongs_to_project", project_id).await?;
        if let Some(pid) = parent_id {
            ensure_one_of_record_ids(&["module"], pid)?;
            self.link(pid, "has_file", fid).await?;
            self.link(fid, "belongs_to_module", pid).await?;
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
            "SELECT ->has_file->file.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files directly linked to a task via belongs_to_task edge.
    pub async fn list_files_by_task(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        self.query_graph_list(
            "SELECT <-belongs_to_task<-file.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
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
        Ok(crate::models::FileContext {
            persona: vec![],
            workflow: vec![],
            file,
            contexts,
        })
    }

    /// Gets full inherited context for a file (Project -> Module -> Submodule -> File).
    pub async fn get_file_context_full(
        &self,
        file_id: &str,
    ) -> anyhow::Result<crate::models::FileContext> {
        let mut ctx = self.get_file_context_node(file_id).await?;

        // Resolve ancestry for this file
        let ancestry = self.resolve_structural_ancestry(file_id).await?;

        for mid in ancestry.module_ids {
            ctx.contexts.extend(self.get_linked_context(&mid).await?);
        }
        for pid in &ancestry.project_ids {
            ctx.contexts.extend(self.get_linked_context(pid).await?);
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

        for pid in &ancestry.project_ids {
            ctx.persona
                .extend(self.list_personas_by_project(pid).await?);
            ctx.workflow
                .extend(self.list_workflows_by_project(pid).await?);
        }

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

    /// Updates a file's name, path, description, or notes.
    pub async fn update_file(
        &self,
        file_id: &str,
        name: Option<String>,
        path: Option<String>,
        description: Option<String>,
        notes: Option<String>,
    ) -> anyhow::Result<Option<crate::models::File>> {
        let key = file_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(file_id);

        let mut patch = serde_json::Map::new();
        if let Some(name) = name {
            patch.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(path) = path {
            patch.insert("path".to_string(), serde_json::Value::String(path));
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
            return self.get_file(file_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("file", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
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
        let table = ensure_one_of_record_ids(&["module"], "module:1")
            .expect("should accept module record id");
        assert_eq!(table, "module");
    }

    #[test]
    fn ensure_one_of_record_ids_rejects_other_tables_or_missing_prefix() {
        let err = ensure_one_of_record_ids(&["module"], "project:1")
            .expect_err("should reject project record id");
        assert!(err.to_string().contains("Expected record id"));

        let err =
            ensure_one_of_record_ids(&["module"], "1").expect_err("should reject missing prefix");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[tokio::test]
    async fn test_list_files_by_task_returns_linked_files() {
        let db = DB::new("mem://").await.expect("init db");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "proj".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();

        let task = db
            .create_task("My Task", "desc", None, Some(&project_id))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        let f1 = db
            .create_file("a.rs", "src/a.rs", None, None, &project_id, None)
            .await
            .expect("create file a");
        let f2 = db
            .create_file("b.rs", "src/b.rs", None, None, &project_id, None)
            .await
            .expect("create file b");
        let f1_id = f1.id.as_ref().unwrap().clone();
        let f2_id = f2.id.as_ref().unwrap().clone();

        db.link(&f1_id, "belongs_to_task", &task_id)
            .await
            .expect("link f1");
        db.link(&f2_id, "belongs_to_task", &task_id)
            .await
            .expect("link f2");

        let files = db
            .list_files_by_task(&task_id)
            .await
            .expect("list files by task");
        assert_eq!(files.len(), 2);
        let ids: Vec<_> = files.iter().map(|f| f.id.as_deref().unwrap()).collect();
        assert!(ids.contains(&f1_id.as_str()));
        assert!(ids.contains(&f2_id.as_str()));
    }

    #[tokio::test]
    async fn test_list_files_by_task_returns_empty_when_no_links() {
        let db = DB::new("mem://").await.expect("init db");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "proj".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();

        let task = db
            .create_task("Empty Task", "desc", None, Some(&project_id))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        // Create a file but do NOT link it to the task
        db.create_file(
            "unlinked.rs",
            "src/unlinked.rs",
            None,
            None,
            &project_id,
            None,
        )
        .await
        .expect("create file");

        let files = db
            .list_files_by_task(&task_id)
            .await
            .expect("list files by task");
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_delete_file_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "test project".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();
        let file = db
            .create_file("delete_me.rs", "path", None, None, &project_id, None)
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
    db: &DB,
) -> anyhow::Result<crate::models::FileContext> {
    db.get_file_context(file_id, full).await
}
