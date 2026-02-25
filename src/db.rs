#[derive(Clone)]
pub struct DB {
    client: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

/// Full structural hierarchy for a node (used when creating reverse knowledge edges).
#[allow(dead_code)]
struct StructuralHierarchy {
    project_id: Option<String>,
    module_id: Option<String>,
    submodule_id: Option<String>,
    task_id: Option<String>,
    subtask_id: Option<String>,
}

impl DB {
    /// TODO: try and unify new methods.
    /// Creates a new SurrealDB client and selects the default namespace/database.
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(url).await?;

        // if is backend is cloud
        if !url.starts_with("mem:") {
            client
                .signin(surrealdb::opt::auth::Root {
                    username: "root".to_string(),
                    password: "root".to_string(),
                })
                .await?;
        }
        client.use_ns("dunno").use_db("dunno").await?;
        let db = Self { client };
        db.define_schema().await?;
        Ok(db)
    }

    /// Creates a DB client from runtime config (local embedded or cloud).
    pub async fn from_config(config: &crate::config::Config) -> anyhow::Result<Self> {
        match config.backend {
            crate::config::StorageBackend::Local => {
                let path = config.local_data_path();
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let url = format!("surrealkv://{}", path.to_string_lossy());
                Self::new_local(&url, "dunno", "dunno").await
            }
            crate::config::StorageBackend::Cloud => {
                let cloud = &config.cloud;
                if cloud.url.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.url` (or DUNNO_CLOUD_URL)"
                    ));
                }
                if cloud.namespace.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.namespace` (or DUNNO_CLOUD_NS)"
                    ));
                }
                if cloud.database.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.database` (or DUNNO_CLOUD_DB)"
                    ));
                }
                if cloud.username.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.username` (or DUNNO_CLOUD_USER)"
                    ));
                }
                if cloud.password.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.password` (or DUNNO_CLOUD_PASS)"
                    ));
                }
                Self::connect_cloud(cloud).await
            }
        }
    }

    async fn new_local(url: &str, namespace: &str, database: &str) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(url).await?;
        if url.starts_with("ws://")
            || url.starts_with("wss://")
            || url.starts_with("http://")
            || url.starts_with("https://")
        {
            client
                .signin(surrealdb::opt::auth::Root {
                    username: "root".to_string(),
                    password: "root".to_string(),
                })
                .await?;
        }
        client.use_ns(namespace).use_db(database).await?;
        let db = Self { client };
        db.define_schema().await?;
        Ok(db)
    }

    async fn connect_cloud(cloud: &crate::config::CloudConfig) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(&cloud.url).await?;
        client
            .use_ns(&cloud.namespace)
            .use_db(&cloud.database)
            .await?;

        match cloud.auth_type.as_str() {
            "namespace" => {
                client
                    .signin(surrealdb::opt::auth::Namespace {
                        namespace: cloud.namespace.clone(),
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
            "database" => {
                client
                    .signin(surrealdb::opt::auth::Database {
                        namespace: cloud.namespace.clone(),
                        database: cloud.database.clone(),
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
            _ => {
                client
                    .signin(surrealdb::opt::auth::Root {
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
        }

        let db = Self { client };
        db.define_schema().await?;
        Ok(db)
    }

    // --- Project Operations ---

    /// Creates a new project record.
    pub async fn create_project(&self, project: &crate::models::Project) -> anyhow::Result<crate::models::Project> {
        let json = serde_json::to_value(project)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("project").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create project"))
        }
    }

    /// Fetches a project by record id.
    pub async fn get_project(&self, id: &str) -> anyhow::Result<Option<crate::models::Project>> {
        self.get_record("project", id).await
    }

    /// Returns all projects.
    pub async fn list_projects(&self) -> anyhow::Result<Vec<crate::models::Project>> {
        self.list_records("project").await
    }

    // --- crate::models::Module Operations ---

    /// Creates a module and RELATEs it to its parent project.
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Module> {
        let module = crate::models::Module {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = serde_json::to_value(&module)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("module").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create module"))?;
        let result: crate::models::Module = serde_json::from_value(surreal_to_json(val))?;
        let module_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::Module missing id after create"))?;

        self.relate(project_id, "contains", module_id).await?;
        Ok(result)
    }

    /// Fetches a module by record id.
    pub async fn get_module(&self, id: &str) -> anyhow::Result<Option<crate::models::Module>> {
        self.get_record("module", id).await
    }

    /// Lists modules under a project via graph traversal.
    pub async fn list_modules_by_project(&self, project_id: &str) -> anyhow::Result<Vec<crate::models::Module>> {
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

    // --- crate::models::Submodule Operations ---

    /// Creates a submodule and RELATEs it to its parent module.
    pub async fn create_submodule(
        &self,
        name: &str,
        description: &str,
        module_id: &str,
    ) -> anyhow::Result<crate::models::Submodule> {
        let submodule = crate::models::Submodule {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = serde_json::to_value(&submodule)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("submodule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create submodule"))?;
        let result: crate::models::Submodule = serde_json::from_value(surreal_to_json(val))?;
        let sub_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::Submodule missing id after create"))?;

        self.relate(module_id, "contains", sub_id).await?;
        Ok(result)
    }

    /// Fetches a submodule by record id.
    pub async fn get_submodule(&self, id: &str) -> anyhow::Result<Option<crate::models::Submodule>> {
        self.get_record("submodule", id).await
    }

    /// Returns all submodules.
    pub async fn list_submodules(&self) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.list_records("submodule").await
    }

    /// Lists submodules under a module via graph traversal.
    pub async fn list_submodules_by_module(&self, module_id: &str) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.query_graph_list(
            "SELECT ->contains->submodule.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    // --- crate::models::File Operations ---

    /// Creates a file and RELATEs it to a parent (module or submodule).
    pub async fn create_file(&self, name: &str, path: &str, parent_id: &str) -> anyhow::Result<crate::models::File> {
        let file = crate::models::File {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
        };
        let json = serde_json::to_value(&file)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("file").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create file"))?;
        let result: crate::models::File = serde_json::from_value(surreal_to_json(val))?;
        let file_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::File missing id after create"))?;

        self.relate(parent_id, "contains", file_id).await?;
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
    pub async fn list_files_by_module(&self, module_id: &str) -> anyhow::Result<Vec<crate::models::File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files under a submodule via graph traversal.
    pub async fn list_files_by_submodule(&self, submodule_id: &str) -> anyhow::Result<Vec<crate::models::File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($sid)",
            "sid",
            submodule_id.to_string(),
            "items",
        )
        .await
    }

    // --- crate::models::Task Operations ---

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
        let created: Option<surrealdb::types::Value> = self.client.create("task").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create task"))?;
        let result: crate::models::Task = serde_json::from_value(surreal_to_json(val))?;
        let task_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::Task missing id after create"))?;

        self.relate(project_id, "has_task", task_id).await?;
        self.relate(task_id, "belongs_to_project", project_id).await?;
        self.relate(task_id, "belongs_to_module", module_id).await?;
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

    /// Lists tasks under a module via graph traversal (tasks that belong to this module).
    pub async fn list_tasks_by_module(&self, module_id: &str) -> anyhow::Result<Vec<crate::models::Task>> {
        self.query_graph_list(
            "SELECT <-belongs_to_module<-task.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists all tasks directly under a project via has_task relationship.
    pub async fn list_tasks_by_project(&self, project_id: &str) -> anyhow::Result<Vec<crate::models::Task>> {
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
    pub async fn get_task_context(&self, task_id: &str) -> anyhow::Result<crate::models::TaskContext> {
        let task = self
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("crate::models::Task not found: {}", task_id))?;

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
    async fn get_task_hierarchy(&self, task_id: &str) -> anyhow::Result<crate::models::TaskHierarchy> {
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
            .ok_or_else(|| anyhow::anyhow!("crate::models::Project missing id"))?;
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
            .ok_or_else(|| anyhow::anyhow!("crate::models::Module missing id"))?;
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

    /// Resolves the full structural hierarchy for a structural node (project, module, submodule, task, subtask).
    /// Used to create reverse belongs_to edges from knowledge nodes to every level in the chain.
    async fn resolve_structural_hierarchy(&self, from_id: &str) -> anyhow::Result<StructuralHierarchy> {
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

    /// Returns the id of the first record in a query result (e.g. first element of alias array).
    async fn first_record_id_from_query(
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
    async fn get_submodule_under_module(&self, task_id: &str) -> anyhow::Result<Option<crate::models::SubmoduleInfo>> {
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
                    .ok_or_else(|| anyhow::anyhow!("crate::models::Submodule missing id"))?
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
    async fn get_files_from_hierarchy(&self, hierarchy: &crate::models::TaskHierarchy) -> anyhow::Result<Vec<String>> {
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
    async fn get_linked_knowledge<T: serde::de::DeserializeOwned + 'static>(
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

    // --- crate::models::Subtask Operations ---

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
        let created: Option<surrealdb::types::Value> = self.client.create("subtask").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create subtask"))?;
        let result: crate::models::Subtask = serde_json::from_value(surreal_to_json(val))?;
        let subtask_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::Subtask missing id after create"))?;

        self.relate(task_id, "has_subtask", subtask_id).await?;
        self.relate(subtask_id, "belongs_to_task", task_id).await?;
        Ok(result)
    }

    /// Fetches a subtask by record id.
    pub async fn get_subtask(&self, id: &str) -> anyhow::Result<Option<crate::models::Subtask>> {
        self.get_record("subtask", id).await
    }

    /// Lists subtasks under a task via has_subtask relationship.
    pub async fn list_subtasks_by_task(&self, task_id: &str) -> anyhow::Result<Vec<crate::models::Subtask>> {
        self.query_graph_list(
            "SELECT ->has_subtask->subtask.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }

    // --- Todo Operations ---

    /// Creates a todo item and RELATEs it to a project via `has_todo`.
    pub async fn create_todo(&self, content: &str, project_id: &str) -> anyhow::Result<crate::models::TodoItem> {
        let todo = crate::models::TodoItem {
            id: None,
            content: content.to_string(),
        };
        let json = serde_json::to_value(&todo)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("todo_item").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create todo item"))?;
        let result: crate::models::TodoItem = serde_json::from_value(surreal_to_json(val))?;
        let todo_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("crate::models::TodoItem missing id after create"))?;

        self.relate(project_id, "has_todo", todo_id).await?;
        Ok(result)
    }

    /// Fetches a todo item by record id.
    pub async fn get_todo(&self, id: &str) -> anyhow::Result<Option<crate::models::TodoItem>> {
        self.get_record("todo_item", id).await
    }

    /// Lists todo items for a project via graph traversal.
    pub async fn list_todos_by_project(&self, project_id: &str) -> anyhow::Result<Vec<crate::models::TodoItem>> {
        self.query_graph_list(
            "SELECT ->has_todo->todo_item.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all todo items (unfiltered).
    pub async fn list_todos(&self) -> anyhow::Result<Vec<crate::models::TodoItem>> {
        self.list_records("todo_item").await
    }

    // --- Knowledge Operations ---

    /// Creates a new mistake record.
    pub async fn create_mistake(&self, mistake: &crate::models::Mistake) -> anyhow::Result<crate::models::Mistake> {
        let json = serde_json::to_value(mistake)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("mistake").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create mistake"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a mistake by record id.
    pub async fn get_mistake(&self, id: &str) -> anyhow::Result<Option<crate::models::Mistake>> {
        self.get_record("mistake", id).await
    }

    /// Returns all mistakes.
    pub async fn list_mistakes(&self) -> anyhow::Result<Vec<crate::models::Mistake>> {
        self.list_records("mistake").await
    }

    /// Creates a new style rule record.
    pub async fn create_style_rule(&self, rule: &crate::models::StyleRule) -> anyhow::Result<crate::models::StyleRule> {
        let json = serde_json::to_value(rule)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("style_rule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create style rule"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a style rule by record id.
    pub async fn get_style_rule(&self, id: &str) -> anyhow::Result<Option<crate::models::StyleRule>> {
        self.get_record("style_rule", id).await
    }

    /// Returns all style rules.
    pub async fn list_style_rules(&self) -> anyhow::Result<Vec<crate::models::StyleRule>> {
        self.list_records("style_rule").await
    }

    /// Creates a new security detail record.
    pub async fn create_security_detail(&self, detail: &crate::models::SecurityDetail) -> anyhow::Result<crate::models::SecurityDetail> {
        let json = serde_json::to_value(detail)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> = self.client.create("security_detail").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create security detail"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a security detail by record id.
    pub async fn get_security_detail(&self, id: &str) -> anyhow::Result<Option<crate::models::SecurityDetail>> {
        self.get_record("security_detail", id).await
    }

    /// Returns all security details.
    pub async fn list_security_details(&self) -> anyhow::Result<Vec<crate::models::SecurityDetail>> {
        self.list_records("security_detail").await
    }

    // --- Graph Edge Operations ---

    /// Creates a knowledge edge from a structural node to a knowledge node.
    /// Uses has_mistake, has_style, or has_security_detail based on to_id prefix.
    /// Also creates reverse edges from the knowledge node using the same relation names
    /// as tasks: belongs_to_project, belongs_to_module, belongs_to_task (for each level present).
    pub async fn link_context(&self, from_id: &str, to_id: &str) -> anyhow::Result<()> {
        let edge = if to_id.starts_with("mistake:") {
            "has_mistake"
        } else if to_id.starts_with("style_rule:") {
            "has_style"
        } else if to_id.starts_with("security_detail:") {
            "has_security_detail"
        } else {
            return Err(anyhow::anyhow!(
                "link_context: to_id must be mistake:, style_rule:, or security_detail: record; got {:?}",
                to_id
            ));
        };
        self.relate(from_id, edge, to_id).await?;

        let hierarchy = self.resolve_structural_hierarchy(from_id).await?;
        if let Some(id) = hierarchy.project_id {
            self.relate(to_id, "belongs_to_project", &id).await?;
        }
        if let Some(id) = hierarchy.module_id {
            self.relate(to_id, "belongs_to_module", &id).await?;
        }
        if let Some(id) = hierarchy.task_id {
            self.relate(to_id, "belongs_to_task", &id).await?;
        }
        Ok(())
    }

    /// Returns all structural node ids that the given knowledge record points to via
    /// belongs_to_project, belongs_to_module, and belongs_to_task (same edges as tasks).
    pub async fn get_belongs_to_targets(&self, knowledge_record_id: &str) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        for (edge, table) in [
            ("belongs_to_project", "project"),
            ("belongs_to_module", "module"),
            ("belongs_to_task", "task"),
        ] {
            let id = self
                .first_record_id_from_query(
                    &format!("SELECT ->{edge}->{table}.* AS out FROM ONLY type::record($kid)"),
                    "kid",
                    knowledge_record_id.to_string(),
                    "out",
                )
                .await?;
            if let Some(id) = id {
                out.push(id);
            }
        }
        Ok(out)
    }

    // --- Maintenance Operations ---

    /// Deletes all records from all tables.
    pub async fn purge_database(&self) -> anyhow::Result<()> {
        let tables = [
            "project",
            "module",
            "submodule",
            "file",
            "task",
            "subtask",
            "todo_item",
            "mistake",
            "style_rule",
            "security_detail",
        ];

        for table in tables {
            let sql = format!("DELETE {}", table);
            self.client.query(&sql).await?;
        }
        Ok(())
    }

    /// Defines relation table schemas so Surrealist can visualize graph edges.
    async fn define_schema(&self) -> anyhow::Result<()> {
        self.client
            .query(
                "\
                DEFINE TABLE IF NOT EXISTS contains TYPE RELATION \
                    IN project|module|submodule \
                    OUT module|submodule|file;\
                DEFINE TABLE IF NOT EXISTS has_task TYPE RELATION \
                    IN project OUT task;\
                DEFINE TABLE IF NOT EXISTS belongs_to_project TYPE RELATION \
                    IN task|mistake|style_rule|security_detail OUT project;\
                DEFINE TABLE IF NOT EXISTS belongs_to_module TYPE RELATION \
                    IN task|mistake|style_rule|security_detail OUT module;\
                DEFINE TABLE IF NOT EXISTS has_subtask TYPE RELATION \
                    IN task OUT subtask;\
                DEFINE TABLE IF NOT EXISTS belongs_to_task TYPE RELATION \
                    IN subtask|mistake|style_rule|security_detail OUT task;\
                DEFINE TABLE IF NOT EXISTS has_mistake TYPE RELATION \
                    IN project|task|module|submodule|subtask OUT mistake;\
                DEFINE TABLE IF NOT EXISTS has_style TYPE RELATION \
                    IN project|task|module|submodule|subtask OUT style_rule;\
                DEFINE TABLE IF NOT EXISTS has_security_detail TYPE RELATION \
                    IN project|task|module|submodule|subtask OUT security_detail;\
                DEFINE TABLE IF NOT EXISTS has_todo TYPE RELATION \
                    IN project OUT todo_item;\
                ",
            )
            .await?;
        Ok(())
    }

    // --- Generic Helpers ---

    /// Runs a raw SurrealQL query with a single string binding.
    /// Returns the result at the given statement index as a serde_json::Value.
    pub async fn query_raw_json(
        &self,
        sql: &str,
        key: &str,
        value: String,
        take_index: usize,
    ) -> anyhow::Result<serde_json::Value> {
        let mut response = self
            .client
            .query(sql)
            .bind((key.to_string(), value))
            .await?;
        let val: surrealdb::types::Value = response.take(take_index)?;
        Ok(surreal_to_json(val))
    }

    /// Creates a RELATE edge between two record ids.
    async fn relate(&self, from_id: &str, edge_table: &str, to_id: &str) -> anyhow::Result<()> {
        let sql = format!(
            "LET $f = type::record($from); \
             LET $t = type::record($to); \
             RELATE $f->{edge_table}->$t;"
        );
        self.client
            .query(&sql)
            .bind(("from", from_id.to_string()))
            .bind(("to", to_id.to_string()))
            .await?;
        Ok(())
    }

    async fn get_record<T: serde::de::DeserializeOwned>(
        &self,
        table: &str,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<surrealdb::types::Value> = match self.client.select((table, key)).await {
            Ok(value) => value,
            Err(err) if is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    async fn list_records<T: serde::de::DeserializeOwned>(&self, table: &str) -> anyhow::Result<Vec<T>> {
        let fetched: Vec<surrealdb::types::Value> = match self.client.select(table).await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Runs a graph traversal query and extracts a nested array.
    ///
    /// SurrealDB graph queries return `{ field: [[...]] }`. This helper
    /// unwraps the outer array and deserializes each inner element.
    async fn query_graph_list<T: serde::de::DeserializeOwned>(
        &self,
        sql: &str,
        bind_key: &str,
        bind_val: String,
        field: &str,
    ) -> anyhow::Result<Vec<T>> {
        let mut response = self
            .client
            .query(sql)
            .bind((bind_key.to_string(), bind_val))
            .await?;
        let row: Option<surrealdb::types::Value> = response.take(0)?;
        let row = match row {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let json = surreal_to_json(row);
        let items = match json.get(field) {
            Some(serde_json::Value::Array(outer)) => {
                let mut flat = Vec::new();
                for elem in outer {
                    match elem {
                        serde_json::Value::Array(inner) => {
                            for item in inner {
                                flat.push(serde_json::from_value(item.clone())?);
                            }
                        }
                        _ => {
                            flat.push(serde_json::from_value(elem.clone())?);
                        }
                    }
                }
                flat
            }
            _ => Vec::new(),
        };
        Ok(items)
    }
}

fn is_missing_table_error(err: &surrealdb::Error) -> bool {
    err.to_string().contains("does not exist")
}

fn json_to_surreal(json: serde_json::Value) -> surrealdb::types::Value {
    match json {
        serde_json::Value::Null => surrealdb::types::Value::Null,
        serde_json::Value::Bool(b) => surrealdb::types::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                surrealdb::types::Value::Number(i.into())
            } else {
                surrealdb::types::Value::Number(n.as_f64().unwrap_or(0.0).into())
            }
        }
        serde_json::Value::String(s) => surrealdb::types::Value::String(s),
        serde_json::Value::Array(a) => surrealdb::types::Value::Array(
            a.into_iter()
                .map(json_to_surreal)
                .collect::<Vec<_>>()
                .into(),
        ),
        serde_json::Value::Object(o) => {
            let mut map = std::collections::BTreeMap::new();
            for (k, v) in o {
                if k == "id" && v.is_null() {
                    continue;
                }
                map.insert(k, json_to_surreal(v));
            }
            surrealdb::types::Value::Object(map.into())
        }
    }
}

fn surreal_to_json(val: surrealdb::types::Value) -> serde_json::Value {
    match val {
        surrealdb::types::Value::None | surrealdb::types::Value::Null => serde_json::Value::Null,
        surrealdb::types::Value::Bool(b) => serde_json::Value::Bool(b),
        surrealdb::types::Value::Number(n) => {
            let s = n.to_string();
            if let Ok(i) = s.parse::<i64>() {
                i.into()
            } else {
                s.parse::<f64>().unwrap_or(0.0).into()
            }
        }
        surrealdb::types::Value::String(s) => s.into(),
        surrealdb::types::Value::Array(a) => serde_json::Value::Array(a.into_iter().map(surreal_to_json).collect()),
        surrealdb::types::Value::Object(o) => {
            let mut map = serde_json::Map::new();
            for (k, v) in o {
                map.insert(k, surreal_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        surrealdb::types::Value::RecordId(t) => {
            let key_debug = format!("{:?}", t.key);
            let key_str = if let Some(inner) = key_debug
                .strip_prefix("String(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                inner.to_string()
            } else {
                key_debug
            };
            serde_json::Value::String(format!("{}:{}", t.table, key_str))
        }
        _ => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CloudConfig, LocalConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_surreal_crud() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mistake = crate::models::Mistake {
            id: None,
            content: "Using unwrap in production code".to_string(),
        };

        let created = db
            .create_mistake(&mistake)
            .await
            .expect("Failed to create mistake");
        assert!(created.id.is_some());
        assert_eq!(created.content, mistake.content);

        let id = created.id.as_ref().unwrap();
        let fetched = db.get_mistake(id).await.expect("Failed to fetch mistake");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().content, mistake.content);
    }

    #[tokio::test]
    async fn test_graph_relate_and_list() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test".to_string(),
                description: "Test project".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("crate::models::Project id should exist");

        let module = db
            .create_module("crate::models::Module", "crate::models::Module desc", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("crate::models::Module id should exist");

        let modules = db
            .list_modules_by_project(&project_id)
            .await
            .expect("Failed to list modules");
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "crate::models::Module");

        let submodule = db
            .create_submodule("Sub", "Sub desc", &module_id)
            .await
            .expect("Failed to create submodule");
        let sub_id = submodule.id.expect("crate::models::Submodule id should exist");

        let subs = db
            .list_submodules_by_module(&module_id)
            .await
            .expect("Failed to list submodules");
        assert_eq!(subs.len(), 1);

        let file = db
            .create_file("test.rs", "src/test.rs", &sub_id)
            .await
            .expect("Failed to create file");
        assert!(file.id.is_some());

        let files = db
            .list_files_by_submodule(&sub_id)
            .await
            .expect("Failed to list files");
        assert_eq!(files.len(), 1);
    }

    #[tokio::test]
    async fn test_link_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Ctx".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("id");

        let module = db
            .create_module("M", "d", &project_id)
            .await
            .expect("create module");
        let module_id = module.id.expect("id");

        let mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "test mistake".to_string(),
            })
            .await
            .expect("create mistake");
        let mistake_id = mistake.id.expect("id");

        db.link_context(&module_id, &mistake_id)
            .await
            .expect("link_context should succeed");
    }

    #[tokio::test]
    async fn test_link_context_all_levels() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.expect("id");

        let module = db
            .create_module("M", "d", &project_id)
            .await
            .expect("create module");
        let module_id = module.id.expect("id");

        let task = db
            .create_task("T", "d", &module_id, &project_id)
            .await
            .expect("create task");
        let task_id = task.id.expect("id");

        let subtask = db
            .create_subtask("ST", "d", &task_id)
            .await
            .expect("create subtask");
        let subtask_id = subtask.id.expect("id");

        let submodule = db
            .create_submodule("SM", "d", &module_id)
            .await
            .expect("create submodule");
        let submodule_id = submodule.id.expect("id");

        let file = db
            .create_file("f.rs", "src/f.rs", &submodule_id)
            .await
            .expect("create file");
        let file_id = file.id.expect("id");

        let project_mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "Project Level Mistake".to_string(),
            })
            .await
            .expect("create mistake");
        db.link_context(&project_id, project_mistake.id.as_ref().unwrap())
            .await
            .expect("link project mistake");

        let submodule_mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "Submodule Level Mistake".to_string(),
            })
            .await
            .expect("create mistake");
        db.link_context(&submodule_id, submodule_mistake.id.as_ref().unwrap())
            .await
            .expect("link submodule mistake");

        let subtask_mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "Subtask Level Mistake".to_string(),
            })
            .await
            .expect("create mistake");
        db.link_context(&subtask_id, subtask_mistake.id.as_ref().unwrap())
            .await
            .expect("link subtask mistake");

        let task_ctx = crate::context::get_task_context(&task_id, &db)
            .await
            .expect("get_task_context");
        assert!(
            task_ctx.iter().any(|v| v["content"] == "Project Level Mistake"),
            "task context should include project-level mistake: {:?}",
            task_ctx
        );

        let file_ctx = crate::context::get_file_context(&file_id, &db)
            .await
            .expect("get_file_context");
        assert!(
            file_ctx.iter().any(|v| v["content"] == "Submodule Level Mistake"),
            "file context should include submodule-level mistake: {:?}",
            file_ctx
        );
        assert!(
            file_ctx.iter().any(|v| v["content"] == "Project Level Mistake"),
            "file context should include project-level mistake: {:?}",
            file_ctx
        );

        let subtask_ctx = crate::context::get_subtask_context(&subtask_id, &db)
            .await
            .expect("get_subtask_context");
        assert!(
            subtask_ctx.iter().any(|v| v["content"] == "Subtask Level Mistake"),
            "subtask context should include subtask-level mistake: {:?}",
            subtask_ctx
        );
        assert!(
            subtask_ctx.iter().any(|v| v["content"] == "Project Level Mistake"),
            "subtask context should include project-level mistake: {:?}",
            subtask_ctx
        );
    }

    #[tokio::test]
    async fn test_link_context_reverse_belongs_to() {
        let db = DB::new("mem://").await.expect("init DB");
        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.expect("id");

        let module = db
            .create_module("M", "d", &project_id)
            .await
            .expect("create module");
        let module_id = module.id.expect("id");

        let task = db
            .create_task("T", "d", &module_id, &project_id)
            .await
            .expect("create task");
        let task_id = task.id.expect("id");

        let mistake1 = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "m1".to_string(),
            })
            .await
            .expect("create mistake");
        let mistake1_id = mistake1.id.expect("id");

        db.link_context(&project_id, &mistake1_id)
            .await
            .expect("link project -> mistake");
        let targets1 = db.get_belongs_to_targets(&mistake1_id).await.expect("get_belongs_to_targets");
        assert!(
            targets1.contains(&project_id),
            "mistake linked to project should have belongs_to project: {:?}",
            targets1
        );

        let mistake2 = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "m2".to_string(),
            })
            .await
            .expect("create mistake");
        let mistake2_id = mistake2.id.expect("id");
        db.link_context(&task_id, &mistake2_id)
            .await
            .expect("link task -> mistake");
        let targets2 = db.get_belongs_to_targets(&mistake2_id).await.expect("get_belongs_to_targets");
        assert!(
            targets2.contains(&project_id),
            "mistake linked to task should belong_to project: {:?}",
            targets2
        );
        assert!(
            targets2.contains(&module_id),
            "mistake linked to task should belong_to module: {:?}",
            targets2
        );
        assert!(
            targets2.contains(&task_id),
            "mistake linked to task should belong_to task: {:?}",
            targets2
        );
    }

    #[tokio::test]
    async fn test_subtask_crud() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "P".to_string(),
                description: "d".to_string(),
            })
            .await
            .unwrap();
        let pid = project.id.unwrap();

        let module = db.create_module("M", "d", &pid).await.unwrap();
        let mid = module.id.unwrap();

        let task = db.create_task("T", "d", &mid, &pid).await.unwrap();
        let tid = task.id.unwrap();

        let subtask = db
            .create_subtask("ST", "sub desc", &tid)
            .await
            .expect("create subtask");
        assert!(subtask.id.is_some());
        assert_eq!(subtask.name, "ST");

        let list = db.list_subtasks_by_task(&tid).await.expect("list subtasks");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "ST");
    }

    #[tokio::test]
    async fn test_from_config_local_embedded_crud() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let db_path = std::env::temp_dir()
            .join(format!("dunno-db-{ts}"))
            .join("data.db");

        let config = crate::config::Config {
            backend: crate::config::StorageBackend::Local,
            local: LocalConfig {
                path: db_path.to_string_lossy().to_string(),
            },
            cloud: CloudConfig::default(),
            qdrant_url: "mem://".to_string(),
        };

        let db = DB::from_config(&config)
            .await
            .expect("local embedded config should surrealdb::engine::any::connect");
        let created = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Embedded".to_string(),
                description: "embedded local test".to_string(),
            })
            .await
            .expect("project create should work");
        assert!(created.id.is_some());

        let _ = cleanup_temp_db(db_path);
    }

    #[tokio::test]
    async fn test_from_config_cloud_validation() {
        let config = crate::config::Config {
            backend: crate::config::StorageBackend::Cloud,
            local: LocalConfig::default(),
            cloud: CloudConfig::default(),
            qdrant_url: "mem://".to_string(),
        };
        let err = match DB::from_config(&config).await {
            Ok(_) => panic!("missing cloud fields should fail"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("cloud.url"));
    }

    fn cleanup_temp_db(db_path: std::path::PathBuf) -> anyhow::Result<()> {
        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
        }
        if let Some(parent) = db_path.parent()
            && parent.exists()
        {
            std::fs::remove_dir_all(parent)?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_task_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("Login", "Implement login", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let _subtask = db
            .create_subtask("Setup DB", "Create tables", &task_id)
            .await
            .expect("Failed to create subtask");

        let mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "crate::models::Task specific mistake".to_string(),
            })
            .await
            .expect("Failed to create mistake");
        db.link_context(&task_id, &mistake.id.unwrap())
            .await
            .expect("Failed to link mistake");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert_eq!(context.task.name, "Login");
        assert_eq!(context.subtasks.len(), 1);
        assert_eq!(context.mistakes.len(), 1);
        assert_eq!(context.hierarchy.project_name, "Testcrate::models::Project");
        assert_eq!(context.hierarchy.module_name, "Auth");
    }

    #[tokio::test]
    async fn test_list_tasks_by_project() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("crate::models::Module1", "First module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let _task1 = db
            .create_task("crate::models::Task1", "First task", &module_id, &project_id)
            .await
            .expect("Failed to create task1");

        let _task2 = db
            .create_task("crate::models::Task2", "Second task", &module_id, &project_id)
            .await
            .expect("Failed to create task2");

        let tasks = db.list_tasks_by_project(&project_id).await.expect("list_tasks_by_project failed");

        assert_eq!(tasks.len(), 2);
        let names: Vec<&str> = tasks.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"crate::models::Task1"));
        assert!(names.contains(&"crate::models::Task2"));
    }

    #[tokio::test]
    async fn test_create_task_bidirectional_edges() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("crate::models::Module1", "First module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("Testcrate::models::Task", "Test task", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let _task_id = task.id.expect("task id");

        let tasks_from_project = db.list_tasks_by_project(&project_id).await.expect("list_tasks_by_project failed");
        assert_eq!(tasks_from_project.len(), 1);

        let tasks_from_module = db.list_tasks_by_module(&module_id).await.expect("list_tasks_by_module failed");
        assert_eq!(tasks_from_module.len(), 1);
    }

    #[tokio::test]
    async fn test_get_task_context_no_linked_knowledge() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("Login", "Implement login", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert_eq!(context.task.name, "Login");
        assert!(context.subtasks.is_empty());
        assert!(context.mistakes.is_empty());
        assert!(context.style_rules.is_empty());
        assert!(context.security_details.is_empty());
    }

    #[tokio::test]
    async fn test_get_task_context_all_knowledge_types() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("Login", "Implement login", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "Using unwrap".to_string(),
            })
            .await
            .expect("Failed to create mistake");
        db.link_context(&task_id, &mistake.id.unwrap()).await.expect("link mistake");

        let style = db
            .create_style_rule(&crate::models::StyleRule {
                id: None,
                description: "Use match".to_string(),
                example: "match".to_string(),
            })
            .await
            .expect("Failed to create style rule");
        db.link_context(&task_id, &style.id.unwrap()).await.expect("link style");

        let security = db
            .create_security_detail(&crate::models::SecurityDetail {
                id: None,
                content: "SQL injection".to_string(),
                severity: "high".to_string(),
                category: "injection".to_string(),
                tags: vec!["sql".to_string()],
            })
            .await
            .expect("Failed to create security detail");
        db.link_context(&task_id, &security.id.unwrap()).await.expect("link security");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert_eq!(context.mistakes.len(), 1);
        assert_eq!(context.style_rules.len(), 1);
        assert_eq!(context.security_details.len(), 1);
        assert_eq!(context.mistakes[0].content, "Using unwrap");
        assert_eq!(context.style_rules[0].description, "Use match");
        assert_eq!(context.security_details[0].severity, "high");
    }

    #[tokio::test]
    async fn test_get_task_context_under_submodule() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let submodule = db
            .create_submodule("JWT", "JWT submodule", &module_id)
            .await
            .expect("Failed to create submodule");
        let _submodule_id = submodule.id.expect("submodule id");

        let task = db
            .create_task("Implement JWT", "Implement JWT auth", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert_eq!(context.hierarchy.module_name, "Auth");
        assert!(context.hierarchy.submodule.is_none());
    }

    #[tokio::test]
    async fn test_get_task_context_files_from_module() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("Login", "Implement login", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert!(context.files.is_empty());
    }

    #[tokio::test]
    async fn test_project_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test description".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let fetched = db.get_project(&project_id).await.expect("get_project failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Testcrate::models::Project");

        let projects = db.list_projects().await.expect("list_projects failed");
        assert_eq!(projects.len(), 1);
    }

    #[tokio::test]
    async fn test_module_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let fetched = db.get_module(&module_id).await.expect("get_module failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Auth");

        let modules = db.list_modules().await.expect("list_modules failed");
        assert_eq!(modules.len(), 1);

        let modules_by_project = db.list_modules_by_project(&project_id).await.expect("list_modules_by_project failed");
        assert_eq!(modules_by_project.len(), 1);
    }

    #[tokio::test]
    async fn test_submodule_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let submodule = db
            .create_submodule("JWT", "JWT submodule", &module_id)
            .await
            .expect("Failed to create submodule");
        let submodule_id = submodule.id.expect("submodule id");

        let fetched = db.get_submodule(&submodule_id).await.expect("get_submodule failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "JWT");

        let submodules = db.list_submodules().await.expect("list_submodules failed");
        assert_eq!(submodules.len(), 1);

        let submodules_by_module = db.list_submodules_by_module(&module_id).await.expect("list_submodules_by_module failed");
        assert_eq!(submodules_by_module.len(), 1);
    }

    #[tokio::test]
    async fn test_file_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let file = db
            .create_file("main.rs", "src/main.rs", &module_id)
            .await
            .expect("Failed to create file");
        let file_id = file.id.expect("file id");

        let fetched = db.get_file(&file_id).await.expect("get_file failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "main.rs");

        let files = db.list_files().await.expect("list_files failed");
        assert_eq!(files.len(), 1);

        let files_by_module = db.list_files_by_module(&module_id).await.expect("list_files_by_module failed");
        assert_eq!(files_by_module.len(), 1);
    }

    #[tokio::test]
    async fn test_todo_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let todo = db
            .create_todo("Buy milk", &project_id)
            .await
            .expect("Failed to create todo");
        let todo_id = todo.id.expect("todo id");

        let fetched = db.get_todo(&todo_id).await.expect("get_todo failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().content, "Buy milk");

        let todos = db.list_todos().await.expect("list_todos failed");
        assert_eq!(todos.len(), 1);

        let todos_by_project = db.list_todos_by_project(&project_id).await.expect("list_todos_by_project failed");
        assert_eq!(todos_by_project.len(), 1);
    }

    #[tokio::test]
    async fn test_list_tasks_by_module() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Testcrate::models::Project".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Auth", "Auth module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let _task = db
            .create_task("crate::models::Task1", "crate::models::Task 1", &module_id, &project_id)
            .await
            .expect("Failed to create task");

        let tasks = db.list_tasks_by_module(&module_id).await.expect("list_tasks_by_module failed");
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_mistake_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mistake = db
            .create_mistake(&crate::models::Mistake {
                id: None,
                content: "Using unwrap".to_string(),
            })
            .await
            .expect("Failed to create mistake");
        let mistake_id = mistake.id.expect("mistake id");

        let fetched = db.get_mistake(&mistake_id).await.expect("get_mistake failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().content, "Using unwrap");

        let mistakes = db.list_mistakes().await.expect("list_mistakes failed");
        assert_eq!(mistakes.len(), 1);
    }

    #[tokio::test]
    async fn test_style_rule_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let rule = db
            .create_style_rule(&crate::models::StyleRule {
                id: None,
                description: "Use match".to_string(),
                example: "match".to_string(),
            })
            .await
            .expect("Failed to create style rule");
        let rule_id = rule.id.expect("rule id");

        let fetched = db.get_style_rule(&rule_id).await.expect("get_style_rule failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().description, "Use match");

        let rules = db.list_style_rules().await.expect("list_style_rules failed");
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_security_detail_operations() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let detail = db
            .create_security_detail(&crate::models::SecurityDetail {
                id: None,
                content: "SQL injection".to_string(),
                severity: "high".to_string(),
                category: "injection".to_string(),
                tags: vec!["sql".to_string()],
            })
            .await
            .expect("Failed to create security detail");
        let detail_id = detail.id.expect("detail id");

        let fetched = db.get_security_detail(&detail_id).await.expect("get_security_detail failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().severity, "high");

        let details = db.list_security_details().await.expect("list_security_details failed");
        assert_eq!(details.len(), 1);
    }

    #[tokio::test]
    async fn test_purge_database() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let _project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Purge Test".to_string(),
                description: "To be purged".to_string(),
            })
            .await
            .expect("Failed to create project");
        
        let projects_before = db.list_projects().await.expect("List projects");
        assert_eq!(projects_before.len(), 1);

        db.purge_database().await.expect("Failed to purge database");

        let projects_after = db.list_projects().await.expect("List projects after purge");
        assert_eq!(projects_after.len(), 0);
    }
}
