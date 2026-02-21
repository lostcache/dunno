use crate::config::{Config, StorageBackend};
use crate::models::{
    File, Mistake, Module, Project, SecurityDetail, StyleRule, Submodule, SubmoduleInfo,
    Subtask, Task, TaskContext, TaskHierarchy, TaskStatus, TaskUpdate, TodoItem,
};
use anyhow::Result;
use serde_json::to_value as to_json_value;
use std::collections::BTreeMap;
use std::fs;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::types::Value;

#[derive(Clone)]
pub struct DB {
    client: Surreal<Any>,
}

impl DB {
    /// TODO: try and unify new methods.
    /// Creates a new SurrealDB client and selects the default namespace/database.
    pub async fn new(url: &str) -> Result<Self> {
        let client = connect(url).await?;

        // if is backend is cloud
        if !url.starts_with("mem:") {
            client
                .signin(surrealdb::opt::auth::Root {
                    username: "root".to_string(),
                    password: "root".to_string(),
                })
                .await?;
        }
        client.use_ns("lazydev").use_db("lazydev").await?;
        Ok(Self { client })
    }

    /// Creates a DB client from runtime config (local embedded or cloud).
    pub async fn from_config(config: &Config) -> Result<Self> {
        match config.backend {
            StorageBackend::Local => {
                let path = config.local_data_path();
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let url = format!("surrealkv://{}", path.to_string_lossy());
                Self::new_local(&url, "lazydev", "lazydev").await
            }
            StorageBackend::Cloud => {
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

    async fn new_local(url: &str, namespace: &str, database: &str) -> Result<Self> {
        let client = connect(url).await?;
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
        Ok(Self { client })
    }

    async fn connect_cloud(cloud: &crate::config::CloudConfig) -> Result<Self> {
        let client = connect(&cloud.url).await?;
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

        Ok(Self { client })
    }

    // --- Project Operations ---

    /// Creates a new project record.
    pub async fn create_project(&self, project: &Project) -> Result<Project> {
        let json = to_json_value(project)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("project").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create project"))
        }
    }

    /// Fetches a project by record id.
    pub async fn get_project(&self, id: &str) -> Result<Option<Project>> {
        self.get_record("project", id).await
    }

    /// Returns all projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.list_records("project").await
    }

    // --- Module Operations ---

    /// Creates a module and RELATEs it to its parent project.
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        project_id: &str,
    ) -> Result<Module> {
        let module = Module {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = to_json_value(&module)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("module").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create module"))?;
        let result: Module = serde_json::from_value(surreal_to_json(val))?;
        let module_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Module missing id after create"))?;

        self.relate(project_id, "contains", module_id).await?;
        Ok(result)
    }

    /// Fetches a module by record id.
    pub async fn get_module(&self, id: &str) -> Result<Option<Module>> {
        self.get_record("module", id).await
    }

    /// Lists modules under a project via graph traversal.
    pub async fn list_modules_by_project(&self, project_id: &str) -> Result<Vec<Module>> {
        self.query_graph_list(
            "SELECT ->contains->module.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all modules (unfiltered).
    pub async fn list_modules(&self) -> Result<Vec<Module>> {
        self.list_records("module").await
    }

    // --- Submodule Operations ---

    /// Creates a submodule and RELATEs it to its parent module.
    pub async fn create_submodule(
        &self,
        name: &str,
        description: &str,
        module_id: &str,
    ) -> Result<Submodule> {
        let submodule = Submodule {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = to_json_value(&submodule)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("submodule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create submodule"))?;
        let result: Submodule = serde_json::from_value(surreal_to_json(val))?;
        let sub_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Submodule missing id after create"))?;

        self.relate(module_id, "contains", sub_id).await?;
        Ok(result)
    }

    /// Fetches a submodule by record id.
    pub async fn get_submodule(&self, id: &str) -> Result<Option<Submodule>> {
        self.get_record("submodule", id).await
    }

    /// Returns all submodules.
    pub async fn list_submodules(&self) -> Result<Vec<Submodule>> {
        self.list_records("submodule").await
    }

    /// Lists submodules under a module via graph traversal.
    pub async fn list_submodules_by_module(&self, module_id: &str) -> Result<Vec<Submodule>> {
        self.query_graph_list(
            "SELECT ->contains->submodule.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    // --- File Operations ---

    /// Creates a file and RELATEs it to a parent (module or submodule).
    pub async fn create_file(&self, name: &str, path: &str, parent_id: &str) -> Result<File> {
        let file = File {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
        };
        let json = to_json_value(&file)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("file").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create file"))?;
        let result: File = serde_json::from_value(surreal_to_json(val))?;
        let file_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("File missing id after create"))?;

        self.relate(parent_id, "contains", file_id).await?;
        Ok(result)
    }

    /// Fetches a file by record id.
    pub async fn get_file(&self, id: &str) -> Result<Option<File>> {
        self.get_record("file", id).await
    }

    /// Returns all files.
    pub async fn list_files(&self) -> Result<Vec<File>> {
        self.list_records("file").await
    }

    /// Lists files under a module via graph traversal.
    pub async fn list_files_by_module(&self, module_id: &str) -> Result<Vec<File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files under a submodule via graph traversal.
    pub async fn list_files_by_submodule(&self, submodule_id: &str) -> Result<Vec<File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($sid)",
            "sid",
            submodule_id.to_string(),
            "items",
        )
        .await
    }

    // --- Task Operations ---

    /// Creates a task and RELATEs it to its parent module with bidirectional edges.
    pub async fn create_task(
        &self,
        name: &str,
        description: &str,
        module_id: &str,
        project_id: &str,
    ) -> Result<Task> {
        let task = Task {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            status: TaskStatus::NotStarted,
        };
        let json = to_json_value(&task)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("task").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create task"))?;
        let result: Task = serde_json::from_value(surreal_to_json(val))?;
        let task_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Task missing id after create"))?;

        self.relate(module_id, "contains", task_id).await?;
        self.relate(project_id, "has_task", task_id).await?;
        self.relate(task_id, "belongs_to_project", project_id).await?;
        self.relate(task_id, "belongs_to_module", module_id).await?;
        Ok(result)
    }

    /// Fetches a task by record id.
    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        self.get_record("task", id).await
    }

    /// Returns all tasks (unfiltered).
    pub async fn list_tasks(&self) -> Result<Vec<Task>> {
        self.list_records("task").await
    }

    /// Lists tasks under a module via graph traversal.
    pub async fn list_tasks_by_module(&self, module_id: &str) -> Result<Vec<Task>> {
        self.query_graph_list(
            "SELECT ->contains->task.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists all tasks directly under a project via has_task relationship.
    pub async fn list_tasks_by_project(&self, project_id: &str) -> Result<Vec<Task>> {
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
        status: Option<TaskStatus>,
    ) -> Result<Option<Task>> {
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

        let updated: Option<Value> = self
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

    /// Gets full context for a task including subtasks, updates, files, and linked knowledge.
    pub async fn get_task_context(&self, task_id: &str) -> Result<TaskContext> {
        let task = self
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        let subtasks = self.list_subtasks_by_task(task_id).await?;
        let updates = self.list_task_updates(task_id).await?;

        let hierarchy = self.get_task_hierarchy(task_id).await?;

        let files = self.get_files_from_hierarchy(&hierarchy).await?;

        let mistakes = self
            .get_linked_knowledge::<Mistake>(task_id, "mistake")
            .await?;
        let style_rules = self
            .get_linked_knowledge::<StyleRule>(task_id, "style_rule")
            .await?;
        let security_details = self
            .get_linked_knowledge::<SecurityDetail>(task_id, "security_detail")
            .await?;

        Ok(TaskContext {
            task,
            subtasks,
            updates,
            files,
            mistakes,
            style_rules,
            security_details,
            hierarchy,
        })
    }

    /// Resolves the hierarchy path from a task to its project/module/submodule.
    async fn get_task_hierarchy(&self, task_id: &str) -> Result<TaskHierarchy> {
        let mut response = self
            .client
            .query("SELECT ->belongs_to_project->project.* AS project FROM ONLY type::record($tid)")
            .bind(("tid", task_id.to_string()))
            .await?;
        let project_record: Option<Value> = response.take(0)?;

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
        let module_record: Option<Value> = response.take(0)?;

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

        Ok(TaskHierarchy {
            project_id: project_id.to_string(),
            project_name,
            module_id: module_id.to_string(),
            module_name,
            submodule,
        })
    }

    /// Gets the submodule if the task belongs to one.
    async fn get_submodule_under_module(&self, task_id: &str) -> Result<Option<SubmoduleInfo>> {
        let mut response = self
            .client
            .query(
                "SELECT ->belongs_to_module->contains->submodule.* AS submodule FROM ONLY type::record($tid)",
            )
            .bind(("tid", task_id.to_string()))
            .await?;
        let result: Option<Value> = response.take(0)?;

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
                return Ok(Some(SubmoduleInfo { id, name }));
            }
        }
        Ok(None)
    }

    /// Gets files from the parent module or submodule in the hierarchy.
    async fn get_files_from_hierarchy(&self, hierarchy: &TaskHierarchy) -> Result<Vec<String>> {
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
    ) -> Result<Vec<T>> {
        let key = task_id.split_once(':').map(|(_, k)| k).unwrap_or(task_id);

        let query = format!(
            "SELECT ->has_context->{}.* AS items FROM ONLY type::record('task', $key)",
            table
        );

        let mut response = self
            .client
            .query(&query)
            .bind(("key", key.to_string()))
            .await?;
        let result: Option<Value> = response.take(0)?;

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

    // --- Subtask Operations ---

    /// Creates a subtask and RELATEs it to its parent task.
    pub async fn create_subtask(
        &self,
        name: &str,
        description: &str,
        task_id: &str,
    ) -> Result<Subtask> {
        let subtask = Subtask {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            status: TaskStatus::NotStarted,
        };
        let json = to_json_value(&subtask)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("subtask").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create subtask"))?;
        let result: Subtask = serde_json::from_value(surreal_to_json(val))?;
        let subtask_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Subtask missing id after create"))?;

        self.relate(task_id, "contains", subtask_id).await?;
        Ok(result)
    }

    /// Fetches a subtask by record id.
    pub async fn get_subtask(&self, id: &str) -> Result<Option<Subtask>> {
        self.get_record("subtask", id).await
    }

    /// Lists subtasks under a task via graph traversal.
    pub async fn list_subtasks_by_task(&self, task_id: &str) -> Result<Vec<Subtask>> {
        self.query_graph_list(
            "SELECT ->contains->subtask.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }

    // --- TaskUpdate Operations ---

    /// Creates a task update and RELATEs it to its parent task via `has_update`.
    pub async fn create_task_update(
        &self,
        content: &str,
        created_at_ms: i64,
        task_id: &str,
    ) -> Result<TaskUpdate> {
        let update = TaskUpdate {
            id: None,
            content: content.to_string(),
            created_at_ms,
            updated_at_ms: None,
        };
        let json = to_json_value(&update)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("task_update").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create task update"))?;
        let result: TaskUpdate = serde_json::from_value(surreal_to_json(val))?;
        let update_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("TaskUpdate missing id after create"))?;

        self.relate(task_id, "has_update", update_id).await?;
        Ok(result)
    }

    /// Lists task updates for a task via graph traversal.
    pub async fn list_task_updates(&self, task_id: &str) -> Result<Vec<TaskUpdate>> {
        let mut updates: Vec<TaskUpdate> = self
            .query_graph_list(
                "SELECT ->has_update->task_update.* AS items FROM ONLY type::record($tid)",
                "tid",
                task_id.to_string(),
                "items",
            )
            .await?;
        updates.sort_by_key(|u| u.created_at_ms);
        Ok(updates)
    }

    /// Edits a task update's content.
    pub async fn update_task_update(
        &self,
        update_id: &str,
        content: String,
        updated_at_ms: i64,
    ) -> Result<Option<TaskUpdate>> {
        let key = update_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(update_id);
        let mut patch = serde_json::Map::new();
        patch.insert("content".to_string(), serde_json::Value::String(content));
        patch.insert(
            "updated_at_ms".to_string(),
            serde_json::Value::Number(updated_at_ms.into()),
        );
        let updated: Option<Value> = self
            .client
            .update(("task_update", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;
        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    // --- Todo Operations ---

    /// Creates a todo item and RELATEs it to a project via `has_todo`.
    pub async fn create_todo(&self, content: &str, project_id: &str) -> Result<TodoItem> {
        let todo = TodoItem {
            id: None,
            content: content.to_string(),
        };
        let json = to_json_value(&todo)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("todo_item").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create todo item"))?;
        let result: TodoItem = serde_json::from_value(surreal_to_json(val))?;
        let todo_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("TodoItem missing id after create"))?;

        self.relate(project_id, "has_todo", todo_id).await?;
        Ok(result)
    }

    /// Fetches a todo item by record id.
    pub async fn get_todo(&self, id: &str) -> Result<Option<TodoItem>> {
        self.get_record("todo_item", id).await
    }

    /// Lists todo items for a project via graph traversal.
    pub async fn list_todos_by_project(&self, project_id: &str) -> Result<Vec<TodoItem>> {
        self.query_graph_list(
            "SELECT ->has_todo->todo_item.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all todo items (unfiltered).
    pub async fn list_todos(&self) -> Result<Vec<TodoItem>> {
        self.list_records("todo_item").await
    }

    // --- Knowledge Operations ---

    /// Creates a new mistake record.
    pub async fn create_mistake(&self, mistake: &Mistake) -> Result<Mistake> {
        let json = to_json_value(mistake)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("mistake").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create mistake"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a mistake by record id.
    pub async fn get_mistake(&self, id: &str) -> Result<Option<Mistake>> {
        self.get_record("mistake", id).await
    }

    /// Returns all mistakes.
    pub async fn list_mistakes(&self) -> Result<Vec<Mistake>> {
        self.list_records("mistake").await
    }

    /// Creates a new style rule record.
    pub async fn create_style_rule(&self, rule: &StyleRule) -> Result<StyleRule> {
        let json = to_json_value(rule)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("style_rule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create style rule"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a style rule by record id.
    pub async fn get_style_rule(&self, id: &str) -> Result<Option<StyleRule>> {
        self.get_record("style_rule", id).await
    }

    /// Returns all style rules.
    pub async fn list_style_rules(&self) -> Result<Vec<StyleRule>> {
        self.list_records("style_rule").await
    }

    /// Creates a new security detail record.
    pub async fn create_security_detail(&self, detail: &SecurityDetail) -> Result<SecurityDetail> {
        let json = to_json_value(detail)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("security_detail").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create security detail"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a security detail by record id.
    pub async fn get_security_detail(&self, id: &str) -> Result<Option<SecurityDetail>> {
        self.get_record("security_detail", id).await
    }

    /// Returns all security details.
    pub async fn list_security_details(&self) -> Result<Vec<SecurityDetail>> {
        self.list_records("security_detail").await
    }

    // --- Graph Edge Operations ---

    /// Creates a `has_context` edge from a structural node to a knowledge node.
    pub async fn link_context(&self, from_id: &str, to_id: &str) -> Result<()> {
        self.relate(from_id, "has_context", to_id).await?;
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
    ) -> Result<serde_json::Value> {
        let mut response = self
            .client
            .query(sql)
            .bind((key.to_string(), value))
            .await?;
        let val: Value = response.take(take_index)?;
        Ok(surreal_to_json(val))
    }

    /// Creates a RELATE edge between two record ids.
    async fn relate(&self, from_id: &str, edge_table: &str, to_id: &str) -> Result<()> {
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
    ) -> Result<Option<T>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<Value> = match self.client.select((table, key)).await {
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

    async fn list_records<T: serde::de::DeserializeOwned>(&self, table: &str) -> Result<Vec<T>> {
        let fetched: Vec<Value> = match self.client.select(table).await {
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
    ) -> Result<Vec<T>> {
        let mut response = self
            .client
            .query(sql)
            .bind((bind_key.to_string(), bind_val))
            .await?;
        let row: Option<Value> = response.take(0)?;
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

fn json_to_surreal(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else {
                Value::Number(n.as_f64().unwrap_or(0.0).into())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(a) => Value::Array(
            a.into_iter()
                .map(json_to_surreal)
                .collect::<Vec<_>>()
                .into(),
        ),
        serde_json::Value::Object(o) => {
            let mut map = BTreeMap::new();
            for (k, v) in o {
                if k == "id" && v.is_null() {
                    continue;
                }
                map.insert(k, json_to_surreal(v));
            }
            Value::Object(map.into())
        }
    }
}

fn surreal_to_json(val: Value) -> serde_json::Value {
    match val {
        Value::None | Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => {
            let s = n.to_string();
            if let Ok(i) = s.parse::<i64>() {
                i.into()
            } else {
                s.parse::<f64>().unwrap_or(0.0).into()
            }
        }
        Value::String(s) => s.into(),
        Value::Array(a) => serde_json::Value::Array(a.into_iter().map(surreal_to_json).collect()),
        Value::Object(o) => {
            let mut map = serde_json::Map::new();
            for (k, v) in o {
                map.insert(k, surreal_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        Value::RecordId(t) => {
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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_surreal_crud() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let mistake = Mistake {
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
            .create_project(&Project {
                id: None,
                name: "Test".to_string(),
                description: "Test project".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("Project id should exist");

        let module = db
            .create_module("Module", "Module desc", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("Module id should exist");

        let modules = db
            .list_modules_by_project(&project_id)
            .await
            .expect("Failed to list modules");
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "Module");

        let submodule = db
            .create_submodule("Sub", "Sub desc", &module_id)
            .await
            .expect("Failed to create submodule");
        let sub_id = submodule.id.expect("Submodule id should exist");

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
    async fn test_task_update_and_append_log() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "Test".to_string(),
                description: "Test project".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("Project id should exist");

        let module = db
            .create_module("Module", "Module desc", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("Module id should exist");

        let task = db
            .create_task("Task", "Initial", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("Task id should exist");

        let updated = db
            .update_task(
                &task_id,
                None,
                Some("Updated description".to_string()),
                Some(TaskStatus::Started),
            )
            .await
            .expect("Failed to update task")
            .expect("Task should exist");
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.status, TaskStatus::Started);

        db.create_task_update("First update", 1, &task_id)
            .await
            .expect("Failed to append first update");
        db.create_task_update("Second update", 2, &task_id)
            .await
            .expect("Failed to append second update");

        let updates = db
            .list_task_updates(&task_id)
            .await
            .expect("Failed to list updates");
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].content, "First update");
        assert_eq!(updates[1].content, "Second update");

        let first_update_id = updates[0]
            .id
            .as_ref()
            .expect("First update should have id")
            .clone();
        let edited = db
            .update_task_update(&first_update_id, "First update (edited)".to_string(), 3)
            .await
            .expect("Failed to edit task update")
            .expect("Task update should exist");
        assert_eq!(edited.content, "First update (edited)");
        assert_eq!(edited.updated_at_ms, Some(3));
    }

    #[tokio::test]
    async fn test_link_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
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
            .create_mistake(&Mistake {
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
    async fn test_subtask_crud() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
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

        let config = Config {
            backend: StorageBackend::Local,
            local: LocalConfig {
                path: db_path.to_string_lossy().to_string(),
            },
            cloud: CloudConfig::default(),
            qdrant_url: "mem://".to_string(),
        };

        let db = DB::from_config(&config)
            .await
            .expect("local embedded config should connect");
        let created = db
            .create_project(&Project {
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
        let config = Config {
            backend: StorageBackend::Cloud,
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

    fn cleanup_temp_db(db_path: PathBuf) -> Result<()> {
        if db_path.exists() {
            fs::remove_file(&db_path)?;
        }
        if let Some(parent) = db_path.parent()
            && parent.exists()
        {
            fs::remove_dir_all(parent)?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_task_context() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
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

        let _update = db
            .create_task_update("Started working", 1000, &task_id)
            .await
            .expect("Failed to create update");

        let mistake = db
            .create_mistake(&Mistake {
                id: None,
                content: "Task specific mistake".to_string(),
            })
            .await
            .expect("Failed to create mistake");
        db.link_context(&task_id, &mistake.id.unwrap())
            .await
            .expect("Failed to link mistake");

        let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

        assert_eq!(context.task.name, "Login");
        assert_eq!(context.subtasks.len(), 1);
        assert_eq!(context.updates.len(), 1);
        assert_eq!(context.mistakes.len(), 1);
        assert_eq!(context.hierarchy.project_name, "TestProject");
        assert_eq!(context.hierarchy.module_name, "Auth");
    }

    #[tokio::test]
    async fn test_list_tasks_by_project() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Module1", "First module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let _task1 = db
            .create_task("Task1", "First task", &module_id, &project_id)
            .await
            .expect("Failed to create task1");

        let _task2 = db
            .create_task("Task2", "Second task", &module_id, &project_id)
            .await
            .expect("Failed to create task2");

        let tasks = db.list_tasks_by_project(&project_id).await.expect("list_tasks_by_project failed");

        assert_eq!(tasks.len(), 2);
        let names: Vec<&str> = tasks.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Task1"));
        assert!(names.contains(&"Task2"));
    }

    #[tokio::test]
    async fn test_create_task_bidirectional_edges() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
                description: "Test".to_string(),
            })
            .await
            .expect("Failed to create project");
        let project_id = project.id.expect("project id");

        let module = db
            .create_module("Module1", "First module", &project_id)
            .await
            .expect("Failed to create module");
        let module_id = module.id.expect("module id");

        let task = db
            .create_task("TestTask", "Test task", &module_id, &project_id)
            .await
            .expect("Failed to create task");
        let task_id = task.id.expect("task id");

        let tasks_from_project = db.list_tasks_by_project(&project_id).await.expect("list_tasks_by_project failed");
        assert_eq!(tasks_from_project.len(), 1);

        let tasks_from_module = db.list_tasks_by_module(&module_id).await.expect("list_tasks_by_module failed");
        assert_eq!(tasks_from_module.len(), 1);
    }

    #[tokio::test]
    async fn test_get_task_context_no_linked_knowledge() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
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
        assert!(context.updates.is_empty());
        assert!(context.mistakes.is_empty());
        assert!(context.style_rules.is_empty());
        assert!(context.security_details.is_empty());
    }

    #[tokio::test]
    async fn test_get_task_context_all_knowledge_types() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let project = db
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
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
            .create_mistake(&Mistake {
                id: None,
                content: "Using unwrap".to_string(),
            })
            .await
            .expect("Failed to create mistake");
        db.link_context(&task_id, &mistake.id.unwrap()).await.expect("link mistake");

        let style = db
            .create_style_rule(&StyleRule {
                id: None,
                description: "Use match".to_string(),
                example: "match".to_string(),
            })
            .await
            .expect("Failed to create style rule");
        db.link_context(&task_id, &style.id.unwrap()).await.expect("link style");

        let security = db
            .create_security_detail(&SecurityDetail {
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
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
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
            .create_project(&Project {
                id: None,
                name: "TestProject".to_string(),
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
}
