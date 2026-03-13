use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

/// Validates task creation parameters.
pub(crate) fn validate_task_params(name: &str, description: &str) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!("Task name cannot be empty"));
    }
    if description.trim().is_empty() {
        return Err(anyhow::anyhow!("Task description cannot be empty"));
    }
    if name.len() > 255 {
        return Err(anyhow::anyhow!("Task name too long (max 255 chars)"));
    }
    Ok(())
}

impl DB {
    /// Internal helper: creates a task record without any relationships.
    pub(crate) async fn create_task_record(
        &self,
        task: &crate::models::Task,
    ) -> anyhow::Result<crate::models::Task> {
        let json = serde_json::to_value(task)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("task").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create task"))?;
        let result: crate::models::Task = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a task and optionally RELATEs it to its parent module and project with bidirectional edges.
    pub async fn create_task(
        &self,
        name: &str,
        description: &str,
        module_id: Option<&str>,
        project_id: Option<&str>,
    ) -> anyhow::Result<crate::models::Task> {
        validate_task_params(name, description)?;

        let task = crate::models::Task {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            status: crate::models::TaskStatus::NotStarted,
        };
        let result = self.create_task_record(&task).await?;

        if let (Some(mid), Some(pid), Some(tid)) = (module_id, project_id, result.id.as_ref()) {
            self.link(pid, "has_task", tid).await?;
            self.link(tid, "belongs_to_project", pid).await?;
            self.link(tid, "belongs_to_module", mid).await?;
        }

        Ok(result)
    }

    /// Fetches a task by record id.
    pub async fn get_task(&self, id: &str) -> anyhow::Result<Option<crate::models::Task>> {
        self.get_record("task", id).await
    }

    /// Returns all tasks (unfiltered).
    pub async fn list_tasks(&self) -> anyhow::Result<Vec<crate::models::Task>> {
        self.list_records("task").await
    }

    /// Lists tasks under a module via graph traversal.
    pub async fn list_tasks_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Task>> {
        self.query_graph_list(
            "SELECT <-belongs_to_module<-task.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists all tasks directly under a project via has_task relationship.
    pub async fn list_tasks_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Task>> {
        self.query_graph_list(
            "SELECT ->has_task->task.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Updates a task's name, description, or status.
    pub async fn update_task(
        &self,
        task_id: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<crate::models::TaskStatus>,
    ) -> anyhow::Result<Option<crate::models::Task>> {
        let key = task_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(task_id);

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
        if let Some(status) = status {
            patch.insert("status".to_string(), serde_json::to_value(status)?);
        }

        if patch.is_empty() {
            return self.get_task(task_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("task", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes a task by record id.
    pub async fn delete_task(&self, task_id: &str) -> anyhow::Result<bool> {
        let key = task_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(task_id);

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("task", key)).await?;
        Ok(deleted.is_some())
    }

    /// Gets context only for the specific task node.
    pub async fn get_task_context_node(
        &self,
        task_id: &str,
    ) -> anyhow::Result<crate::models::TaskContext> {
        let task = self
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        let hierarchy = self.get_task_hierarchy(task_id).await?;
        let files = self.get_files_from_hierarchy(&hierarchy).await?;
        let contexts = self.get_linked_context(task_id).await?;

        Ok(crate::models::TaskContext {
            task,
            files,
            contexts,
            hierarchy,
        })
    }

    /// Gets full inherited context for a task (Project -> Module -> Submodule -> Task).
    pub async fn get_task_context_full(
        &self,
        task_id: &str,
    ) -> anyhow::Result<crate::models::TaskContext> {
        let mut ctx = self.get_task_context_node(task_id).await?;

        if let Some(sub) = &ctx.hierarchy.submodule {
            ctx.contexts.extend(self.get_linked_context(&sub.id).await?);
        }
        ctx.contexts
            .extend(self.get_linked_context(&ctx.hierarchy.module_id).await?);
        ctx.contexts
            .extend(self.get_linked_context(&ctx.hierarchy.project_id).await?);

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

    /// Gets context for a task, optionally including parent hierarchy.
    pub async fn get_task_context(
        &self,
        task_id: &str,
        full: bool,
    ) -> anyhow::Result<crate::models::TaskContext> {
        if full {
            self.get_task_context_full(task_id).await
        } else {
            self.get_task_context_node(task_id).await
        }
    }

    /// Resolves the hierarchy path from a task to its project/module/submodule.
    pub(crate) async fn get_task_hierarchy(
        &self,
        task_id: &str,
    ) -> anyhow::Result<crate::models::TaskHierarchy> {
        let mut response = self
            .client
            .query("SELECT ->belongs_to_project->project.* AS project FROM ONLY type::record($tid)")
            .bind(("tid", task_id.to_string()))
            .await?;
        let project_record: Option<surrealdb::types::Value> = response.take(0)?;

        let project_json = surreal_to_json(
            project_record.ok_or_else(|| anyhow::anyhow!("No project linked to task"))?,
        );
        let project_obj = project_json
            .get("project")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse project from graph query"))?;

        let project_id = project_obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Project missing id"))?;
        let project_name = project_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut response = self
            .client
            .query("SELECT ->belongs_to_module->module.* AS module FROM ONLY type::record($tid)")
            .bind(("tid", task_id.to_string()))
            .await?;
        let module_record: Option<surrealdb::types::Value> = response.take(0)?;

        let module_json = surreal_to_json(
            module_record.ok_or_else(|| anyhow::anyhow!("No module linked to task"))?,
        );
        let module_obj = module_json
            .get("module")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse module from graph query"))?;

        let module_id = module_obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Module missing id"))?;
        let module_name = module_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let submodule = self.get_submodule_under_module(task_id).await?;

        Ok(crate::models::TaskHierarchy {
            project_id: project_id.to_string(),
            project_name,
            module_id: module_id.to_string(),
            module_name,
            submodule,
        })
    }

    /// Gets the submodule if the task belongs to one.
    pub(crate) async fn get_submodule_under_module(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Option<crate::models::SubmoduleInfo>> {
        let mut response = self
            .client
            .query(
                "SELECT ->belongs_to_submodule->submodule.* AS submodule FROM ONLY type::record($tid)",
            )
            .bind(("tid", task_id.to_string()))
            .await?;
        let result: Option<surrealdb::types::Value> = response.take(0)?;
        let json = surreal_to_json(result.ok_or_else(|| anyhow::anyhow!("Query failed"))?);
        if let Some(serde_json::Value::Array(arr)) = json.get("submodule") {
            if let Some(sub) = arr.first() {
                let id = sub
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Submodule missing id"))?
                    .to_string();
                let name = sub
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(Some(crate::models::SubmoduleInfo { id, name }));
            }
        }
        Ok(None)
    }

    /// Gets files from the parent module or submodule in the hierarchy.
    pub(crate) async fn get_files_from_hierarchy(
        &self,
        hierarchy: &crate::models::TaskHierarchy,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        if let Some(ref submodule) = hierarchy.submodule {
            let files = self.list_files_by_submodule(&submodule.id).await?;
            return Ok(files);
        }

        let files = self.list_files_by_module(&hierarchy.module_id).await?;
        Ok(files)
    }

    /// Fetches all context records linked to a structural node (project, task, module, submodule).
    pub(crate) async fn get_linked_context(
        &self,
        structural_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Context>> {
        self.query_graph_list(
            "SELECT ->has_context->context.* AS items FROM ONLY type::record($sid)",
            "sid",
            structural_id.to_string(),
            "items",
        )
        .await
    }
}

/// Returns full task context including task details, files, hierarchy, and linked knowledge.
pub async fn get_task_context_json(
    task_id: &str,
    full: bool,
    db: &crate::db::DB,
) -> anyhow::Result<crate::models::TaskContext> {
    db.get_task_context(task_id, full).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_task_params_accepts_valid_input() {
        validate_task_params("Valid Name", "Valid Description")
            .expect("should accept valid params");
    }

    #[test]
    fn validate_task_params_rejects_empty_name() {
        let err = validate_task_params("", "Description").expect_err("empty name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_task_params_rejects_whitespace_only_name() {
        let err =
            validate_task_params("   ", "Description").expect_err("whitespace name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_task_params_rejects_empty_description() {
        let err = validate_task_params("Name", "").expect_err("empty description should fail");
        assert!(err.to_string().contains("description"));
    }

    #[test]
    fn validate_task_params_rejects_long_name() {
        let long_name = "a".repeat(256);
        let err =
            validate_task_params(&long_name, "Description").expect_err("long name should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn validate_task_params_rejects_max_length_name_plus_one() {
        let long_name = "a".repeat(256);
        let err =
            validate_task_params(&long_name, "Description").expect_err("256 char name should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn validate_task_params_rejects_whitespace_only_description() {
        let err =
            validate_task_params("Name", "   ").expect_err("whitespace description should fail");
        assert!(err.to_string().contains("description"));
    }

    #[tokio::test]
    async fn test_delete_task_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", None, Some(&project_id))
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task(
                "Task to Delete",
                "Will be deleted",
                Some(&module_id),
                Some(&project_id),
            )
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        // Verify task exists
        let fetched = db.get_task(&task_id).await.expect("Failed to fetch task");
        assert!(fetched.is_some());

        // Delete the task
        let deleted = db
            .delete_task(&task_id)
            .await
            .expect("Failed to delete task");
        assert!(deleted, "delete_task should return true for existing task");

        // Verify task no longer exists
        let after_delete = db.get_task(&task_id).await.expect("Failed to check task");
        assert!(after_delete.is_none(), "Task should be deleted");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_task() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        // First create a task to ensure the table exists
        let task = db
            .create_task("Test Task", "For table creation", None, None)
            .await
            .expect("create task");
        let task_id = task.id.expect("task id");

        // Now try to delete a different task that doesn't exist
        let deleted = db
            .delete_task("task:nonexistent12345")
            .await
            .expect("Should not error on nonexistent task when table exists");

        assert!(
            !deleted,
            "delete_task should return false for nonexistent task"
        );

        // Cleanup - delete the task we created
        db.delete_task(&task_id).await.expect("cleanup delete");
    }

    #[tokio::test]
    async fn test_delete_task_with_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", None, Some(&project_id))
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task(
                "Task with Context",
                "Has linked knowledge",
                Some(&module_id),
                Some(&project_id),
            )
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        // Add context to the task
        let ctx = db
            .create_context(&crate::models::Context {
                id: None,
                context_type: "mistake".to_string(),
                content: Some("Context to be deleted".to_string()),
                description: None,
                example: None,
                severity: None,
                category: None,
                tags: None,
            })
            .await
            .expect("Failed to create context");
        let ctx_id = ctx.id.expect("context id");

        db.link_context(&task_id, &ctx_id)
            .await
            .expect("Failed to link context");

        // Verify context is linked
        let contexts_before = db
            .get_linked_context(&task_id)
            .await
            .expect("Failed to get contexts");
        assert_eq!(contexts_before.len(), 1);

        // Delete the task
        let deleted = db
            .delete_task(&task_id)
            .await
            .expect("Failed to delete task");
        assert!(deleted);

        // Verify task is deleted
        let after_delete = db.get_task(&task_id).await.expect("Failed to check task");
        assert!(after_delete.is_none());
    }
}
