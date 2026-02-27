//! Integration tests for SurrealDB backend.
use super::*;
use crate::config::{CloudConfig, LocalConfig};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_surreal_crud() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let ctx = crate::models::Context {
        id: None,
        context_type: "mistake".to_string(),
        content: Some("Using unwrap in production code".to_string()),
        description: None,
        example: None,
        severity: None,
        category: None,
        tags: None,
    };

    let created = db
        .create_context(&ctx)
        .await
        .expect("Failed to create context");
    assert!(created.id.is_some());
    assert_eq!(created.content, ctx.content);

    let id = created.id.as_ref().unwrap();
    let fetched = db.get_context(id).await.expect("Failed to fetch context");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content, ctx.content);
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

    let project_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Project Level Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    db.link_context(&project_id, project_ctx.id.as_ref().unwrap())
        .await
        .expect("link project context");

    let submodule_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Submodule Level Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    db.link_context(&submodule_id, submodule_ctx.id.as_ref().unwrap())
        .await
        .expect("link submodule context");

    let subtask_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Subtask Level Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    db.link_context(&subtask_id, subtask_ctx.id.as_ref().unwrap())
        .await
        .expect("link subtask context");

    let task_ctx = crate::context::get_task_context(&task_id, &db)
        .await
        .expect("get_task_context");
    assert!(
        task_ctx.is_empty(),
        "task context should be task-only (no inherited project context): {:?}",
        task_ctx
    );

    let file_ctx = crate::context::get_file_context(&file_id, &db)
        .await
        .expect("get_file_context");
    assert!(
        file_ctx.is_empty(),
        "file context should be file-only (no inherited submodule/project context): {:?}",
        file_ctx
    );

    // For subtask, verify direct-only context via the DB helper.
    let subtask_ctx = db
        .get_linked_context(&subtask_id)
        .await
        .expect("get_linked_context for subtask");
    assert!(
        subtask_ctx
            .iter()
            .any(|c| c.context_type == "mistake" && c.content.as_deref() == Some("Subtask Level Mistake")),
        "subtask linked context should include subtask-level mistake: {:?}",
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

    let ctx1 = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("m1".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    let ctx1_id = ctx1.id.expect("id");

    db.link_context(&project_id, &ctx1_id)
        .await
        .expect("link project -> context");
    let targets1 = db.get_belongs_to_targets(&ctx1_id).await.expect("get_belongs_to_targets");
    assert!(
        targets1.contains(&project_id),
        "context linked to project should have belongs_to project: {:?}",
        targets1
    );

    let ctx2 = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("m2".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    let ctx2_id = ctx2.id.expect("id");
    db.link_context(&task_id, &ctx2_id)
        .await
        .expect("link task -> context");
    let targets2 = db.get_belongs_to_targets(&ctx2_id).await.expect("get_belongs_to_targets");
    assert!(
        targets2.contains(&project_id),
        "context linked to task should belong_to project: {:?}",
        targets2
    );
    assert!(
        targets2.contains(&module_id),
        "context linked to task should belong_to module: {:?}",
        targets2
    );
    assert!(
        targets2.contains(&task_id),
        "context linked to task should belong_to task: {:?}",
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

    let ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("crate::models::Task specific mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create context");
    db.link_context(&task_id, &ctx.id.unwrap())
        .await
        .expect("Failed to link context");

    let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

    assert_eq!(context.task.name, "Login");
    assert_eq!(context.subtasks.len(), 1);
    assert_eq!(context.contexts.len(), 1);
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
    assert!(context.contexts.is_empty());
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

    let mistake_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Using unwrap".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create context");
    db.link_context(&task_id, &mistake_ctx.id.unwrap()).await.expect("link mistake");

    let style_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "style_rule".to_string(),
            content: None,
            description: Some("Use match".to_string()),
            example: Some("match".to_string()),
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create context");
    db.link_context(&task_id, &style_ctx.id.unwrap()).await.expect("link style");

    let security_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "security_detail".to_string(),
            content: Some("SQL injection".to_string()),
            description: None,
            example: None,
            severity: Some("high".to_string()),
            category: Some("injection".to_string()),
            tags: Some(vec!["sql".to_string()]),
        })
        .await
        .expect("Failed to create context");
    db.link_context(&task_id, &security_ctx.id.unwrap()).await.expect("link security");

    let context = db.get_task_context(&task_id).await.expect("get_task_context failed");

    assert_eq!(context.contexts.len(), 3);
    assert!(context
        .contexts
        .iter()
        .any(|c| c.context_type == "mistake" && c.content.as_deref() == Some("Using unwrap")));
    assert!(context.contexts.iter().any(|c| c.context_type == "style_rule"
        && c.description.as_deref() == Some("Use match")));
    assert!(context.contexts.iter().any(|c| c.context_type == "security_detail"
        && c.severity.as_deref() == Some("high")));
}

#[tokio::test]
async fn test_get_task_context_under_submodule() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
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
    let _submodule = db
        .create_submodule("JWT", "JWT submodule", &module_id)
        .await
        .expect("Failed to create submodule");
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

#[tokio::test]
async fn test_project_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "TestProject".to_string(),
            description: "Test description".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id.expect("project id");
    let fetched = db.get_project(&project_id).await.expect("get_project failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "TestProject");
    let projects = db.list_projects().await.expect("list_projects failed");
    assert_eq!(projects.len(), 1);
}

#[tokio::test]
async fn test_module_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
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
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id.expect("project id");
    let todo = db.create_todo("Buy milk", &project_id).await.expect("Failed to create todo");
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
    let _task = db
        .create_task("Task1", "Task 1", &module_id, &project_id)
        .await
        .expect("Failed to create task");
    let tasks = db.list_tasks_by_module(&module_id).await.expect("list_tasks_by_module failed");
    assert_eq!(tasks.len(), 1);
}

#[tokio::test]
async fn test_mistake_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Using unwrap".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create context");
    let context_id = ctx.id.expect("context id");
    let fetched = db.get_context(&context_id).await.expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content.as_deref(), Some("Using unwrap"));
    let contexts = db.list_contexts_by_type("mistake").await.expect("list_contexts_by_type failed");
    assert_eq!(contexts.len(), 1);
}

#[tokio::test]
async fn test_style_rule_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "style_rule".to_string(),
            content: None,
            description: Some("Use match".to_string()),
            example: Some("match".to_string()),
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create context");
    let context_id = ctx.id.expect("context id");
    let fetched = db.get_context(&context_id).await.expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().description.as_deref(), Some("Use match"));
    let contexts = db.list_contexts_by_type("style_rule").await.expect("list_contexts_by_type failed");
    assert_eq!(contexts.len(), 1);
}

#[tokio::test]
async fn test_security_detail_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "security_detail".to_string(),
            content: Some("SQL injection".to_string()),
            description: None,
            example: None,
            severity: Some("high".to_string()),
            category: Some("injection".to_string()),
            tags: Some(vec!["sql".to_string()]),
        })
        .await
        .expect("Failed to create context");
    let context_id = ctx.id.expect("context id");
    let fetched = db.get_context(&context_id).await.expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().severity.as_deref(), Some("high"));
    let contexts = db.list_contexts_by_type("security_detail").await.expect("list_contexts_by_type failed");
    assert_eq!(contexts.len(), 1);
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
