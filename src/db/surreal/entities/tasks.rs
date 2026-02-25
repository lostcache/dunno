use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::{StructuralHierarchy, DB};

impl DB {
    /// Creates a task and RELATEs it to its parent module with bidirectional edges.
    pub async fn create_task(
        &self,
        name: &str,
        description: &str,
        module_id: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Task> {
        let task = crate::models::Task {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            status: crate::models::TaskStatus::NotStarted,
        };
        let json = serde_json::to_value(&task)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("task").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create task"))?;
        let result: crate::models::Task = serde_json::from_value(surreal_to_json(val))?;
        let task_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Task missing id after create"))?;

        self.relate(project_id, "has_task", task_id).await?;
        self.relate(task_id, "belongs_to_project", project_id).await?;
        self.relate(task_id, "belongs_to_module", module_id).await?;
        Ok(result)
    }

    /// Fetches a task by record id.
    pub async fn get_task(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Task>> {
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

    /// Gets full context for a task including subtasks, files, and linked knowledge.
    pub async fn get_task_context(
        &self,
        task_id: &str,
    ) -> anyhow::Result<crate::models::TaskContext> {
        let task = self
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        let subtasks = self.list_subtasks_by_task(task_id).await?;
        let hierarchy = self.get_task_hierarchy(task_id).await?;
        let files = self.get_files_from_hierarchy(&hierarchy).await?;

        let mistakes = self
            .get_linked_knowledge::<crate::models::Mistake>(task_id, "mistake")
            .await?;
        let style_rules = self
            .get_linked_knowledge::<crate::models::StyleRule>(task_id, "style_rule")
            .await?;
        let security_details = self
            .get_linked_knowledge::<crate::models::SecurityDetail>(task_id, "security_detail")
            .await?;

        Ok(crate::models::TaskContext {
            task,
            subtasks,
            files,
            mistakes,
            style_rules,
            security_details,
            hierarchy,
        })
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

    /// Resolves the full structural hierarchy for a structural node.
    pub(crate) async fn resolve_structural_hierarchy(
        &self,
        from_id: &str,
    ) -> anyhow::Result<StructuralHierarchy> {
        let table = from_id.split_once(':').map(|(t, _)| t).unwrap_or("");
        match table {
            "project" => Ok(StructuralHierarchy {
                project_id: Some(from_id.to_string()),
                module_id: None,
                submodule_id: None,
                task_id: None,
                subtask_id: None,
            }),
            "module" => {
                let project_id = self
                    .first_record_id_from_query(
                        "SELECT <-contains<-project AS p FROM ONLY type::record($mid)",
                        "mid",
                        from_id.to_string(),
                        "p",
                    )
                    .await?;
                Ok(StructuralHierarchy {
                    project_id: project_id.clone(),
                    module_id: Some(from_id.to_string()),
                    submodule_id: None,
                    task_id: None,
                    subtask_id: None,
                })
            }
            "submodule" => {
                let module_id = self
                    .first_record_id_from_query(
                        "SELECT <-contains<-module AS m FROM ONLY type::record($sid)",
                        "sid",
                        from_id.to_string(),
                        "m",
                    )
                    .await?;
                let project_id = match &module_id {
                    Some(mid) => self
                        .first_record_id_from_query(
                            "SELECT <-contains<-project AS p FROM ONLY type::record($mid)",
                            "mid",
                            mid.clone(),
                            "p",
                        )
                        .await?,
                    None => None,
                };
                Ok(StructuralHierarchy {
                    project_id,
                    module_id,
                    submodule_id: Some(from_id.to_string()),
                    task_id: None,
                    subtask_id: None,
                })
            }
            "task" => {
                let hierarchy = self.get_task_hierarchy(from_id).await?;
                let submodule_id = hierarchy.submodule.as_ref().map(|s| s.id.clone());
                Ok(StructuralHierarchy {
                    project_id: Some(hierarchy.project_id),
                    module_id: Some(hierarchy.module_id),
                    submodule_id,
                    task_id: Some(from_id.to_string()),
                    subtask_id: None,
                })
            }
            "subtask" => {
                let task_id = self
                    .first_record_id_from_query(
                        "SELECT ->belongs_to_task->task AS t FROM ONLY type::record($stid)",
                        "stid",
                        from_id.to_string(),
                        "t",
                    )
                    .await?;
                let (project_id, module_id, submodule_id) = match &task_id {
                    Some(tid) => {
                        let h = self.get_task_hierarchy(tid).await?;
                        let sub = h.submodule.as_ref().map(|s| s.id.clone());
                        (
                            Some(h.project_id),
                            Some(h.module_id),
                            sub,
                        )
                    }
                    None => (None, None, None),
                };
                Ok(StructuralHierarchy {
                    project_id,
                    module_id,
                    submodule_id,
                    task_id,
                    subtask_id: Some(from_id.to_string()),
                })
            }
            _ => Err(anyhow::anyhow!(
                "resolve_structural_hierarchy: from_id must be project, module, submodule, task, or subtask; got {:?}",
                table
            )),
        }
    }

    /// Returns the id of the first record in a query result.
    pub(crate) async fn first_record_id_from_query(
        &self,
        sql: &str,
        bind_key: &str,
        bind_val: String,
        result_alias: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut response = self
            .client
            .query(sql)
            .bind((bind_key.to_string(), bind_val))
            .await?;
        let row: Option<surrealdb::types::Value> = response.take(0)?;
        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };
        let json = surreal_to_json(row);
        let id = json
            .get(result_alias)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from);
        Ok(id)
    }

    /// Gets the submodule if the task belongs to one.
    pub(crate) async fn get_submodule_under_module(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Option<crate::models::SubmoduleInfo>> {
        let mut response = self
            .client
            .query(
                "SELECT ->belongs_to_module->contains->submodule.* AS submodule FROM ONLY type::record($tid)",
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
    ) -> anyhow::Result<Vec<String>> {
        if let Some(ref submodule) = hierarchy.submodule {
            let submodule_record = self.get_submodule(&submodule.id).await?;
            if let Some(sub) = submodule_record {
                return Ok(sub.files.unwrap_or_default());
            }
        }

        let module_record = self.get_module(&hierarchy.module_id).await?;
        if let Some(module) = module_record {
            return Ok(module.files.unwrap_or_default());
        }

        Ok(vec![])
    }

    /// Generic method to fetch linked knowledge of a specific type.
    pub(crate) async fn get_linked_knowledge<T: serde::de::DeserializeOwned + 'static>(
        &self,
        task_id: &str,
        table: &str,
    ) -> anyhow::Result<Vec<T>> {
        let key = task_id.split_once(':').map(|(_, k)| k).unwrap_or(task_id);
        let edge = match table {
            "mistake" => "has_mistake",
            "style_rule" => "has_style",
            "security_detail" => "has_security_detail",
            _ => return Err(anyhow::anyhow!("get_linked_knowledge: unknown table {}", table)),
        };
        let query = format!(
            "SELECT ->{}->{}.* AS items FROM ONLY type::record('task', $key)",
            edge, table
        );

        let mut response = self
            .client
            .query(&query)
            .bind(("key", key.to_string()))
            .await?;
        let result: Option<surrealdb::types::Value> = response.take(0)?;

        let json = match result {
            Some(val) => surreal_to_json(val),
            None => return Ok(vec![]),
        };

        if let Some(serde_json::Value::Array(outer)) = json.get("items") {
            let mut items = Vec::new();
            for elem in outer {
                if let serde_json::Value::Array(inner) = elem {
                    for item in inner {
                        items.push(serde_json::from_value(item.clone())?);
                    }
                } else {
                    items.push(serde_json::from_value(elem.clone())?);
                }
            }
            return Ok(items);
        }

        Ok(vec![])
    }

    /// Creates a subtask and RELATEs it to its parent task.
    pub async fn create_subtask(
        &self,
        name: &str,
        description: &str,
        task_id: &str,
    ) -> anyhow::Result<crate::models::Subtask> {
        let subtask = crate::models::Subtask {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            status: crate::models::TaskStatus::NotStarted,
        };
        let json = serde_json::to_value(&subtask)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("subtask").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create subtask"))?;
        let result: crate::models::Subtask = serde_json::from_value(surreal_to_json(val))?;
        let subtask_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Subtask missing id after create"))?;

        self.relate(task_id, "has_subtask", subtask_id).await?;
        self.relate(subtask_id, "belongs_to_task", task_id).await?;
        Ok(result)
    }

    /// Fetches a subtask by record id.
    pub async fn get_subtask(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Subtask>> {
        self.get_record("subtask", id).await
    }

    /// Lists subtasks under a task via has_subtask relationship.
    pub async fn list_subtasks_by_task(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Subtask>> {
        self.query_graph_list(
            "SELECT ->has_subtask->subtask.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }
}
