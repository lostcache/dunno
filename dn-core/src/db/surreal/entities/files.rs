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
    /// Creates a file, links it to a project (required), and optionally RELATEs it to a parent module.
    pub async fn create_files(
        &self,
        names: Vec<String>,
        paths: Vec<String>,
        descriptions: Vec<String>,
        project_id: String,
        parent_mod_ids: Vec<String>,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        let tx = self.client.clone().begin().await?;
        let mut results: Vec<crate::models::File> = vec![];

        for (n, (p, (d, pm))) in std::iter::zip(
            names,
            std::iter::zip(paths, std::iter::zip(descriptions, parent_mod_ids)),
        ) {
            let file_res = async {
                let pid = surrealdb::types::RecordId::parse_simple(&project_id)?;
                let file = crate::models::File {
                    id: None,
                    name: n,
                    path: p,
                    description: if !d.is_empty() { Some(d) } else { None },
                };
                let json = serde_json::to_value(&file)?;
                let surreal_val = json_to_surreal(json);

                let created_file_surreal_val_maybe = tx.create("file").content(surreal_val).await?;

                match created_file_surreal_val_maybe {
                    Some(created_file_surreal_val) => {
                        let created_file: crate::models::File =
                            serde_json::from_value(surreal_to_json(created_file_surreal_val))?;
                        let fid = created_file
                            .id
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("File created without ID"))?;

                        let fid = surrealdb::types::RecordId::parse_simple(fid)?;

                        tx.query("RELATE $from->belongs_to_project->$to")
                            .bind(("from", fid.clone()))
                            .bind(("to", pid.clone()))
                            .await?;

                        tx.query("RELATE $from->has_file->$to")
                            .bind(("from", pid))
                            .bind(("to", fid.clone()))
                            .await?;

                        if !pm.is_empty() {
                            let pmid = surrealdb::types::RecordId::parse_simple(&pm)?;
                            tx.query("RELATE $from->belongs_to_module->$to")
                                .bind(("from", fid.clone()))
                                .bind(("to", pmid.clone()))
                                .await?;

                            tx.query("RELATE $from->has_file->$to")
                                .bind(("from", pmid))
                                .bind(("to", fid))
                                .await?;
                        }

                        Ok(created_file)
                    }
                    None => anyhow::bail!("couldn't create files/s"),
                }
            };

            match file_res.await {
                Ok(file) => results.push(file),
                Err(e) => {
                    tx.cancel().await?;
                    anyhow::bail!("Failed to create file/s: {}", e)
                }
            }
        }

        tx.commit().await?;

        Ok(results)
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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "proj".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id;

        let task = db
            .create_task("My Task", "desc", None, Some(&project_id))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        let f1 = db
            .create_files(
                vec!["a.rs".to_string()],
                vec!["src/a.rs".to_string()],
                vec!["".to_string()],
                project_id.clone(),
                vec!["".to_string()],
            )
            .await
            .expect("create file a");
        let f2 = db
            .create_files(
                vec!["b.rs".to_string()],
                vec!["src/b.rs".to_string()],
                vec!["".to_string()],
                project_id.clone(),
                vec!["".to_string()],
            )
            .await
            .expect("create file b");
        let f1_id = f1[0].id.as_ref().unwrap().clone();
        let f2_id = f2[0].id.as_ref().unwrap().clone();

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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "proj".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id;

        let task = db
            .create_task("Empty Task", "desc", None, Some(&project_id))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        // Create a file but do NOT link it to the task
        db.create_files(
            vec!["unlinked.rs".to_string()],
            vec!["src/unlinked.rs".to_string()],
            vec!["".to_string()],
            project_id,
            vec!["".to_string()],
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
                // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
                // Use `String::new()` as a placeholder when creating a new project.
                id: String::new(),
                name: "test project".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id;
        let file = db
            .create_files(
                vec!["delete_me.rs".to_string()],
                vec!["path".to_string()],
                vec!["".to_string()],
                project_id,
                vec!["".to_string()],
            )
            .await
            .expect("create");
        let id = file[0].id.clone().unwrap();

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
