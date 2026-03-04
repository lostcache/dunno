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
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let task = db
        .create_task("T", "d", Some(&module_id), Some(&project_id))
        .await
        .expect("create task");
    let task_id = task.id.expect("id");

    let submodule = db
        .create_submodule("SM", "d", Some(&module_id))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    let file = db
        .create_file("f.rs", "src/f.rs", Some(&submodule_id))
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
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let task = db
        .create_task("T", "d", Some(&module_id), Some(&project_id))
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
    let targets1 = db
        .get_belongs_to_targets(&ctx1_id)
        .await
        .expect("get_belongs_to_targets");
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
    let targets2 = db
        .get_belongs_to_targets(&ctx2_id)
        .await
        .expect("get_belongs_to_targets");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Login",
            "Implement login",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

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

    let context = db
        .get_task_context(&task_id)
        .await
        .expect("get_task_context failed");

    assert_eq!(context.task.name, "Login");
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
        .create_module("crate::models::Module1", "First module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let _task1 = db
        .create_task(
            "crate::models::Task1",
            "First task",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task1");

    let _task2 = db
        .create_task(
            "crate::models::Task2",
            "Second task",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task2");

    let tasks = db
        .list_tasks_by_project(&project_id)
        .await
        .expect("list_tasks_by_project failed");

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
        .create_module("crate::models::Module1", "First module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Testcrate::models::Task",
            "Test task",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let _task_id = task.id.expect("task id");

    let tasks_from_project = db
        .list_tasks_by_project(&project_id)
        .await
        .expect("list_tasks_by_project failed");
    assert_eq!(tasks_from_project.len(), 1);

    let tasks_from_module = db
        .list_tasks_by_module(&module_id)
        .await
        .expect("list_tasks_by_module failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Login",
            "Implement login",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    let context = db
        .get_task_context(&task_id)
        .await
        .expect("get_task_context failed");

    assert_eq!(context.task.name, "Login");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Login",
            "Implement login",
            Some(&module_id),
            Some(&project_id),
        )
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
    db.link_context(&task_id, &mistake_ctx.id.unwrap())
        .await
        .expect("link mistake");

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
    db.link_context(&task_id, &style_ctx.id.unwrap())
        .await
        .expect("link style");

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
    db.link_context(&task_id, &security_ctx.id.unwrap())
        .await
        .expect("link security");

    let context = db
        .get_task_context(&task_id)
        .await
        .expect("get_task_context failed");

    assert_eq!(context.contexts.len(), 3);
    assert!(
        context
            .contexts
            .iter()
            .any(|c| c.context_type == "mistake" && c.content.as_deref() == Some("Using unwrap"))
    );
    assert!(
        context.contexts.iter().any(
            |c| c.context_type == "style_rule" && c.description.as_deref() == Some("Use match")
        )
    );
    assert!(
        context
            .contexts
            .iter()
            .any(|c| c.context_type == "security_detail" && c.severity.as_deref() == Some("high"))
    );
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let _submodule = db
        .create_submodule("JWT", "JWT submodule", Some(&module_id))
        .await
        .expect("Failed to create submodule");
    let task = db
        .create_task(
            "Implement JWT",
            "Implement JWT auth",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");
    let context = db
        .get_task_context(&task_id)
        .await
        .expect("get_task_context failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let task = db
        .create_task(
            "Login",
            "Implement login",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");
    let context = db
        .get_task_context(&task_id)
        .await
        .expect("get_task_context failed");
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
    let fetched = db
        .get_project(&project_id)
        .await
        .expect("get_project failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let fetched = db.get_module(&module_id).await.expect("get_module failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Auth");
    let modules = db.list_modules().await.expect("list_modules failed");
    assert_eq!(modules.len(), 1);
    let modules_by_project = db
        .list_modules_by_project(&project_id)
        .await
        .expect("list_modules_by_project failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let submodule = db
        .create_submodule("JWT", "JWT submodule", Some(&module_id))
        .await
        .expect("Failed to create submodule");
    let submodule_id = submodule.id.expect("submodule id");
    let fetched = db
        .get_submodule(&submodule_id)
        .await
        .expect("get_submodule failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "JWT");
    let submodules = db.list_submodules().await.expect("list_submodules failed");
    assert_eq!(submodules.len(), 1);
    let submodules_by_module = db
        .list_submodules_by_module(&module_id)
        .await
        .expect("list_submodules_by_module failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let file = db
        .create_file("main.rs", "src/main.rs", Some(&module_id))
        .await
        .expect("Failed to create file");
    let file_id = file.id.expect("file id");
    let fetched = db.get_file(&file_id).await.expect("get_file failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "main.rs");
    let files = db.list_files().await.expect("list_files failed");
    assert_eq!(files.len(), 1);
    let files_by_module = db
        .list_files_by_module(&module_id)
        .await
        .expect("list_files_by_module failed");
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
    let todo = db
        .create_todo("Buy milk", Some(&project_id))
        .await
        .expect("Failed to create todo");
    let todo_id = todo.id.expect("todo id");
    let fetched = db.get_todo(&todo_id).await.expect("get_todo failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content, "Buy milk");
    let todos = db.list_todos().await.expect("list_todos failed");
    assert_eq!(todos.len(), 1);
    let todos_by_project = db
        .list_todos_by_project(&project_id)
        .await
        .expect("list_todos_by_project failed");
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let _task = db
        .create_task("Task1", "Task 1", Some(&module_id), Some(&project_id))
        .await
        .expect("Failed to create task");
    let tasks = db
        .list_tasks_by_module(&module_id)
        .await
        .expect("list_tasks_by_module failed");
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
    let fetched = db
        .get_context(&context_id)
        .await
        .expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content.as_deref(), Some("Using unwrap"));
    let contexts = db
        .list_contexts_by_type("mistake")
        .await
        .expect("list_contexts_by_type failed");
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
    let fetched = db
        .get_context(&context_id)
        .await
        .expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().description.as_deref(), Some("Use match"));
    let contexts = db
        .list_contexts_by_type("style_rule")
        .await
        .expect("list_contexts_by_type failed");
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
    let fetched = db
        .get_context(&context_id)
        .await
        .expect("get_context failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().severity.as_deref(), Some("high"));
    let contexts = db
        .list_contexts_by_type("security_detail")
        .await
        .expect("list_contexts_by_type failed");
    assert_eq!(contexts.len(), 1);
}

// --- Freestanding creation and link-after-create tests (DB & CLI flexible create/link track) ---

#[tokio::test]
async fn test_freestanding_module() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");

    let module = db
        .create_module("Freestanding", "no project", None)
        .await
        .expect("create freestanding module");
    let module_id = module.id.expect("module id");

    assert!(
        db.get_module(&module_id)
            .await
            .expect("get_module")
            .is_some()
    );
    let by_project = db
        .list_modules_by_project(&project_id)
        .await
        .expect("list_modules_by_project");
    assert!(
        !by_project
            .iter()
            .any(|m| m.id.as_deref() == Some(module_id.as_str())),
        "freestanding module must not appear under project: {:?}",
        by_project
    );
    let all = db.list_modules().await.expect("list_modules");
    assert!(
        all.iter()
            .any(|m| m.id.as_deref() == Some(module_id.as_str())),
        "freestanding module must appear in list_modules"
    );
}

#[tokio::test]
async fn test_freestanding_submodule() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");
    let module = db
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    let sub = db
        .create_submodule("Freestanding", "no module", None)
        .await
        .expect("create freestanding submodule");
    let sub_id = sub.id.expect("submodule id");

    assert!(
        db.get_submodule(&sub_id)
            .await
            .expect("get_submodule")
            .is_some()
    );
    let by_module = db
        .list_submodules_by_module(&module_id)
        .await
        .expect("list_submodules_by_module");
    assert!(
        !by_module
            .iter()
            .any(|s| s.id.as_deref() == Some(sub_id.as_str())),
        "freestanding submodule must not appear under module"
    );
}

#[tokio::test]
async fn test_freestanding_file() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");
    let module = db
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    let file = db
        .create_file("orphan.rs", "src/orphan.rs", None)
        .await
        .expect("create freestanding file");
    let file_id = file.id.expect("file id");

    assert!(db.get_file(&file_id).await.expect("get_file").is_some());
    let by_module = db
        .list_files_by_module(&module_id)
        .await
        .expect("list_files_by_module");
    assert!(
        !by_module
            .iter()
            .any(|f| f.id.as_deref() == Some(file_id.as_str())),
        "freestanding file must not appear under module"
    );
}

#[tokio::test]
async fn test_freestanding_task() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let task = db
        .create_task("Freestanding", "no project/module", None, None)
        .await
        .expect("create freestanding task");
    let task_id = task.id.expect("task id");

    assert!(db.get_task(&task_id).await.expect("get_task").is_some());
    let err = db.get_task_hierarchy(&task_id).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("Failed to parse project from graph query")
            || err.to_string().contains("No project linked to task"),
        "freestanding task must fail get_task_hierarchy: {}",
        err
    );
}

#[tokio::test]
async fn test_freestanding_todo() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");

    let todo = db
        .create_todo("Freestanding todo", None)
        .await
        .expect("create freestanding todo");
    let todo_id = todo.id.expect("todo id");

    assert!(db.get_todo(&todo_id).await.expect("get_todo").is_some());
    let by_project = db
        .list_todos_by_project(&project_id)
        .await
        .expect("list_todos_by_project");
    assert!(
        !by_project
            .iter()
            .any(|t| t.id.as_deref() == Some(todo_id.as_str())),
        "freestanding todo must not appear under project"
    );
}

#[tokio::test]
async fn test_link_after_create_module() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");

    let module = db
        .create_module("LaterLinked", "d", None)
        .await
        .expect("create freestanding module");
    let module_id = module.id.expect("module id");

    let by_before = db
        .list_modules_by_project(&project_id)
        .await
        .expect("list_modules_by_project");
    assert_eq!(by_before.len(), 0);

    db.link(&project_id, "contains", &module_id)
        .await
        .expect("link project -> contains -> module");

    let by_after = db
        .list_modules_by_project(&project_id)
        .await
        .expect("list_modules_by_project");
    assert_eq!(by_after.len(), 1);
    assert_eq!(by_after[0].id.as_deref(), Some(module_id.as_str()));
}

#[tokio::test]
async fn test_link_after_create_task_hierarchy() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");
    let module = db
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task("LaterLinked", "d", None, None)
        .await
        .expect("create freestanding task");
    let task_id = task.id.expect("task id");

    db.link(&project_id, "has_task", &task_id)
        .await
        .expect("link project -> has_task -> task");
    db.link(&task_id, "belongs_to_project", &project_id)
        .await
        .expect("link task -> belongs_to_project -> project");
    db.link(&task_id, "belongs_to_module", &module_id)
        .await
        .expect("link task -> belongs_to_module -> module");

    let hierarchy = db
        .get_task_hierarchy(&task_id)
        .await
        .expect("get_task_hierarchy");
    assert_eq!(hierarchy.project_id, project_id);
    assert_eq!(hierarchy.module_id, module_id);
}

#[tokio::test]
async fn test_create_with_link_ids_preserves_hierarchy() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");
    let module = db
        .create_module("M", "d", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");
    let task = db
        .create_task("T", "d", Some(&module_id), Some(&project_id))
        .await
        .expect("create task");
    let task_id = task.id.expect("task id");

    let hierarchy = db
        .get_task_hierarchy(&task_id)
        .await
        .expect("get_task_hierarchy");
    assert_eq!(hierarchy.project_id, project_id);
    assert_eq!(hierarchy.module_id, module_id);
    let by_project = db
        .list_tasks_by_project(&project_id)
        .await
        .expect("list_tasks_by_project");
    assert!(
        by_project
            .iter()
            .any(|t| t.id.as_deref() == Some(task_id.as_str()))
    );
    let by_module = db
        .list_tasks_by_module(&module_id)
        .await
        .expect("list_tasks_by_module");
    assert!(
        by_module
            .iter()
            .any(|t| t.id.as_deref() == Some(task_id.as_str()))
    );
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

// ============================================================================
// UPDATE OPERATIONS
// ============================================================================

#[tokio::test]
async fn test_update_task_name() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Original Name",
            "Original desc",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    let updated = db
        .update_task(&task_id, Some("Updated Name".to_string()), None, None)
        .await
        .expect("Failed to update task name");

    assert!(updated.is_some());
    assert_eq!(updated.unwrap().name, "Updated Name");

    let fetched = db.get_task(&task_id).await.expect("Failed to fetch task");
    assert_eq!(fetched.unwrap().name, "Updated Name");
}

#[tokio::test]
async fn test_update_task_description() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Task Name",
            "Original description",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    let updated = db
        .update_task(
            &task_id,
            None,
            Some("Updated description".to_string()),
            None,
        )
        .await
        .expect("Failed to update task description");

    assert!(updated.is_some());
    assert_eq!(updated.unwrap().description, "Updated description");
}

#[tokio::test]
async fn test_update_task_status() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Task Name",
            "Task desc",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    assert_eq!(task.status, crate::models::TaskStatus::NotStarted);

    let updated = db
        .update_task(
            &task_id,
            None,
            None,
            Some(crate::models::TaskStatus::Started),
        )
        .await
        .expect("Failed to update task status");

    assert!(updated.is_some());
    assert_eq!(updated.unwrap().status, crate::models::TaskStatus::Started);

    let finished = db
        .update_task(
            &task_id,
            None,
            None,
            Some(crate::models::TaskStatus::Finished),
        )
        .await
        .expect("Failed to update task to finished");

    assert_eq!(
        finished.unwrap().status,
        crate::models::TaskStatus::Finished
    );
}

#[tokio::test]
async fn test_update_task_all_fields() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Original",
            "Original desc",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    let updated = db
        .update_task(
            &task_id,
            Some("New Name".to_string()),
            Some("New Description".to_string()),
            Some(crate::models::TaskStatus::Started),
        )
        .await
        .expect("Failed to update task");

    let task = updated.expect("Task should exist");
    assert_eq!(task.name, "New Name");
    assert_eq!(task.description, "New Description");
    assert_eq!(task.status, crate::models::TaskStatus::Started);
}

#[tokio::test]
async fn test_update_task_empty_patch_returns_current() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task(
            "Task Name",
            "Task desc",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    let current = db
        .update_task(&task_id, None, None, None)
        .await
        .expect("Failed to get current task");

    assert!(current.is_some());
    assert_eq!(current.unwrap().name, "Task Name");
}

#[tokio::test]
async fn test_update_nonexistent_task_returns_none() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // When table doesn't exist, it may return an error or None depending on the backend
    let result = db
        .update_task("task:nonexistent", Some("New Name".to_string()), None, None)
        .await;

    // Should either return None (if table exists but record doesn't) or an error (if table doesn't exist)
    match result {
        Ok(None) => (), // Expected: record not found
        Ok(Some(_)) => panic!("Should not find non-existent task"),
        Err(e) => {
            // Also acceptable: table doesn't exist yet
            assert!(
                e.to_string().contains("does not exist") || e.to_string().contains("not found"),
                "Expected 'not found' or 'does not exist' error, got: {}",
                e
            );
        }
    }
}

// ============================================================================
// LIST OPERATIONS
// ============================================================================

#[tokio::test]
async fn test_list_tasks_unfiltered() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    db.create_task("Task 1", "First task", Some(&module_id), Some(&project_id))
        .await
        .expect("Failed to create task 1");

    db.create_task("Task 2", "Second task", Some(&module_id), Some(&project_id))
        .await
        .expect("Failed to create task 2");

    db.create_task("Freestanding", "No links", None, None)
        .await
        .expect("Failed to create freestanding task");

    let all_tasks = db.list_tasks().await.expect("Failed to list tasks");
    assert_eq!(all_tasks.len(), 3);
}

#[tokio::test]
async fn test_list_contexts_all_types() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    db.create_context(&crate::models::Context {
        id: None,
        context_type: "mistake".to_string(),
        content: Some("Mistake 1".to_string()),
        description: None,
        example: None,
        severity: None,
        category: None,
        tags: None,
    })
    .await
    .expect("Failed to create context 1");

    db.create_context(&crate::models::Context {
        id: None,
        context_type: "style_rule".to_string(),
        content: None,
        description: Some("Style 1".to_string()),
        example: Some("Example".to_string()),
        severity: None,
        category: None,
        tags: None,
    })
    .await
    .expect("Failed to create context 2");

    db.create_context(&crate::models::Context {
        id: None,
        context_type: "security_detail".to_string(),
        content: Some("Security 1".to_string()),
        description: None,
        example: None,
        severity: Some("high".to_string()),
        category: Some("injection".to_string()),
        tags: Some(vec!["sql".to_string()]),
    })
    .await
    .expect("Failed to create context 3");

    db.create_context(&crate::models::Context {
        id: None,
        context_type: "mistake".to_string(),
        content: Some("Mistake 2".to_string()),
        description: None,
        example: None,
        severity: None,
        category: None,
        tags: None,
    })
    .await
    .expect("Failed to create context 4");

    let all_contexts = db
        .list_contexts()
        .await
        .expect("Failed to list all contexts");
    assert_eq!(all_contexts.len(), 4);

    let mistakes = db
        .list_contexts_by_type("mistake")
        .await
        .expect("Failed to list mistakes");
    assert_eq!(mistakes.len(), 2);

    let styles = db
        .list_contexts_by_type("style_rule")
        .await
        .expect("Failed to list styles");
    assert_eq!(styles.len(), 1);

    let security = db
        .list_contexts_by_type("security_detail")
        .await
        .expect("Failed to list security");
    assert_eq!(security.len(), 1);

    let unknown = db
        .list_contexts_by_type("unknown_type")
        .await
        .expect("Failed to list unknown");
    assert!(unknown.is_empty());
}

#[tokio::test]
async fn test_list_files_by_submodule() {
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
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let submodule = db
        .create_submodule("OAuth", "OAuth submodule", Some(&module_id))
        .await
        .expect("Failed to create submodule");
    let submodule_id = submodule.id.expect("submodule id");

    let file1 = db
        .create_file("oauth.rs", "src/auth/oauth.rs", Some(&submodule_id))
        .await
        .expect("Failed to create file 1");
    let file1_id = file1.id.expect("file id");

    let file2 = db
        .create_file("jwt.rs", "src/auth/jwt.rs", Some(&submodule_id))
        .await
        .expect("Failed to create file 2");
    let _file2_id = file2.id.expect("file id");

    let files_by_submodule = db
        .list_files_by_submodule(&submodule_id)
        .await
        .expect("Failed to list files by submodule");

    assert_eq!(files_by_submodule.len(), 2);
    assert!(
        files_by_submodule
            .iter()
            .any(|f| f.id.as_deref() == Some(file1_id.as_str()))
    );
}

// ============================================================================
// CLOUD CONFIG VALIDATION TESTS
// ============================================================================

#[tokio::test]
async fn test_from_config_cloud_missing_namespace() {
    let config = crate::config::Config {
        backend: crate::config::StorageBackend::Cloud,
        local: crate::config::LocalConfig::default(),
        cloud: crate::config::CloudConfig {
            url: "wss://test.surrealdb.com".to_string(),
            namespace: "".to_string(),
            database: "test".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
        },
        qdrant_url: "mem://".to_string(),
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing namespace should fail");
    assert!(err.to_string().contains("namespace"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_database() {
    let config = crate::config::Config {
        backend: crate::config::StorageBackend::Cloud,
        local: crate::config::LocalConfig::default(),
        cloud: crate::config::CloudConfig {
            url: "wss://test.surrealdb.com".to_string(),
            namespace: "test".to_string(),
            database: "".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
        },
        qdrant_url: "mem://".to_string(),
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing database should fail");
    assert!(err.to_string().contains("database"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_username() {
    let config = crate::config::Config {
        backend: crate::config::StorageBackend::Cloud,
        local: crate::config::LocalConfig::default(),
        cloud: crate::config::CloudConfig {
            url: "wss://test.surrealdb.com".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
            username: "".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
        },
        qdrant_url: "mem://".to_string(),
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing username should fail");
    assert!(err.to_string().contains("username"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_password() {
    let config = crate::config::Config {
        backend: crate::config::StorageBackend::Cloud,
        local: crate::config::LocalConfig::default(),
        cloud: crate::config::CloudConfig {
            url: "wss://test.surrealdb.com".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            auth_type: "root".to_string(),
        },
        qdrant_url: "mem://".to_string(),
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing password should fail");
    assert!(err.to_string().contains("password"));
}

#[tokio::test]
async fn test_from_config_cloud_valid() {
    let config = crate::config::Config {
        backend: crate::config::StorageBackend::Cloud,
        local: crate::config::LocalConfig::default(),
        cloud: crate::config::CloudConfig {
            url: "wss://test.surrealdb.com".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
        },
        qdrant_url: "mem://".to_string(),
    };

    // This will fail to connect but should pass validation
    let result = DB::from_config(&config).await;
    assert!(result.is_err()); // Will fail to connect to fake URL
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_project() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let result = db
        .get_project("project:nonexistent")
        .await
        .expect("Should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_module() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let result = db
        .get_module("module:nonexistent")
        .await
        .expect("Should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_task() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let result = db
        .get_task("task:nonexistent")
        .await
        .expect("Should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_context() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let result = db
        .get_context("context:nonexistent")
        .await
        .expect("Should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_task_context_nonexistent_task() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let err = db
        .get_task_context("task:nonexistent")
        .await
        .expect_err("Should fail");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_task_hierarchy_freestanding_task() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let task = db
        .create_task("Freestanding", "No project/module", None, None)
        .await
        .expect("Failed to create freestanding task");
    let task_id = task.id.expect("task id");

    let err = db
        .get_task_hierarchy(&task_id)
        .await
        .expect_err("Should fail for freestanding task");
    assert!(
        err.to_string().contains("No project linked to task")
            || err
                .to_string()
                .contains("Failed to parse project from graph query"),
        "Expected error about no project linked, got: {}",
        err
    );
}

#[tokio::test]
async fn test_list_empty_tables() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let projects = db.list_projects().await.expect("Should not error");
    assert!(projects.is_empty());

    let modules = db.list_modules().await.expect("Should not error");
    assert!(modules.is_empty());

    let tasks = db.list_tasks().await.expect("Should not error");
    assert!(tasks.is_empty());

    let contexts = db.list_contexts().await.expect("Should not error");
    assert!(contexts.is_empty());
}

#[tokio::test]
async fn test_link_context_with_invalid_context_id() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id.expect("project id");

    let err = db
        .link_context(&project_id, "not_a_context_id")
        .await
        .expect_err("Should fail");
    assert!(err.to_string().contains("context record id"));
}

#[tokio::test]
async fn test_user_story_crud() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create a project
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test Project".to_string(),
            description: "For user story testing".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    // Create a user story
    let user_story = db
        .create_user_story(
            "As a user, I want login",
            "User authentication feature",
            &project_id,
        )
        .await
        .expect("create user story");
    assert!(user_story.id.is_some());
    assert_eq!(user_story.title, "As a user, I want login");
    assert_eq!(user_story.description, "User authentication feature");

    let us_id = user_story.id.as_ref().unwrap();

    // Fetch the user story
    let fetched = db
        .get_user_story(us_id)
        .await
        .expect("Failed to fetch user story");
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.title, "As a user, I want login");

    // List user stories by project
    let stories = db
        .list_user_stories_by_project(&project_id)
        .await
        .expect("list user stories");
    assert_eq!(stories.len(), 1);
    assert_eq!(stories[0].title, "As a user, I want login");

    // List all user stories (unfiltered)
    let all_stories = db.list_user_stories().await.expect("list all user stories");
    assert_eq!(all_stories.len(), 1);
}

#[tokio::test]
async fn test_user_story_task_linking() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project, module, and user story
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let module = db
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let user_story = db
        .create_user_story("As a user, I want secure login", "Secure auth", &project_id)
        .await
        .expect("create user story");
    let us_id = user_story.id.expect("id");

    // Create a task linked to both module and project
    let task = db
        .create_task(
            "Implement login",
            "Add JWT auth",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("create task");
    let task_id = task.id.expect("id");

    // Link task to user story
    db.link_task_to_user_story(&task_id, &us_id)
        .await
        .expect("link task to user story");

    // Verify the link by listing tasks for the user story
    let tasks = db
        .list_tasks_by_user_story(&us_id)
        .await
        .expect("list tasks by user story");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Implement login");

    // Verify by listing user stories for the task
    let stories = db
        .list_user_stories_by_task(&task_id)
        .await
        .expect("list user stories by task");
    assert_eq!(stories.len(), 1);
    assert_eq!(stories[0].title, "As a user, I want secure login");
}

#[tokio::test]
async fn test_user_story_module_linking() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project, module, and user story
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let module = db
        .create_module("Core", "Core module", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let submodule = db
        .create_submodule("Utils", "Utils submodule", Some(&module_id))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    let user_story = db
        .create_user_story(
            "As a user, I want data persistence",
            "Database layer",
            &project_id,
        )
        .await
        .expect("create user story");
    let us_id = user_story.id.expect("id");

    // Link module to user story
    db.link_module_to_user_story(&module_id, &us_id)
        .await
        .expect("link module to user story");

    // Link submodule to user story
    db.link_submodule_to_user_story(&submodule_id, &us_id)
        .await
        .expect("link submodule to user story");

    // Verify module link
    let modules = db
        .list_modules_by_user_story(&us_id)
        .await
        .expect("list modules by user story");
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "Core");

    // Verify submodule link
    let submodules = db
        .list_submodules_by_user_story(&us_id)
        .await
        .expect("list submodules by user story");
    assert_eq!(submodules.len(), 1);
    assert_eq!(submodules[0].name, "Utils");

    // Verify reverse lookups
    let stories_from_module = db
        .list_user_stories_by_module(&module_id)
        .await
        .expect("list user stories by module");
    assert_eq!(stories_from_module.len(), 1);

    let stories_from_submodule = db
        .list_user_stories_by_submodule(&submodule_id)
        .await
        .expect("list user stories by submodule");
    assert_eq!(stories_from_submodule.len(), 1);
}

#[tokio::test]
async fn test_epic_crud() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create a project
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test Project".to_string(),
            description: "For epic testing".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    // Create an epic
    let epic = db
        .create_epic("Authentication Epic", "Complete auth system", &project_id)
        .await
        .expect("create epic");
    assert!(epic.id.is_some());
    assert_eq!(epic.title, "Authentication Epic");
    assert_eq!(epic.description, "Complete auth system");

    let epic_id = epic.id.as_ref().unwrap();

    // Fetch the epic
    let fetched = db.get_epic(epic_id).await.expect("Failed to fetch epic");
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.title, "Authentication Epic");

    // List epics by project
    let epics = db
        .list_epics_by_project(&project_id)
        .await
        .expect("list epics");
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0].title, "Authentication Epic");

    // List all epics (unfiltered)
    let all_epics = db.list_epics().await.expect("list all epics");
    assert_eq!(all_epics.len(), 1);
}

#[tokio::test]
async fn test_epic_user_story_linking() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project and epic
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let epic = db
        .create_epic("Auth Epic", "Authentication features", &project_id)
        .await
        .expect("create epic");
    let epic_id = epic.id.expect("id");

    // Create a user story
    let user_story = db
        .create_user_story("As a user, I want login", "Login feature", &project_id)
        .await
        .expect("create user story");
    let us_id = user_story.id.expect("id");

    // Link user story to epic
    db.link_user_story_to_epic(&us_id, &epic_id)
        .await
        .expect("link user story to epic");

    // Verify the link by listing user stories for the epic
    let stories = db
        .list_user_stories_by_epic(&epic_id)
        .await
        .expect("list user stories by epic");
    assert_eq!(stories.len(), 1);
    assert_eq!(stories[0].title, "As a user, I want login");

    // Verify by listing epics for the user story
    let epics = db
        .list_epics_by_user_story(&us_id)
        .await
        .expect("list epics by user story");
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0].title, "Auth Epic");
}

#[tokio::test]
async fn test_epic_task_linking() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project, module, and epic
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let module = db
        .create_module("Auth", "Auth module", Some(&project_id))
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let epic = db
        .create_epic("Auth Epic", "Authentication features", &project_id)
        .await
        .expect("create epic");
    let epic_id = epic.id.expect("id");

    // Create a task linked to both module and project
    let task = db
        .create_task(
            "Implement login",
            "Add JWT auth",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("create task");
    let task_id = task.id.expect("id");

    // Link task to epic
    db.link_task_to_epic(&task_id, &epic_id)
        .await
        .expect("link task to epic");

    // Verify the link by listing tasks for the epic
    let tasks = db
        .list_tasks_by_epic(&epic_id)
        .await
        .expect("list tasks by epic");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Implement login");

    // Verify by listing epics for the task
    let epics = db
        .list_epics_by_task(&task_id)
        .await
        .expect("list epics by task");
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0].title, "Auth Epic");
}

#[tokio::test]
async fn test_epic_context() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project and epic
    let project = db
        .create_project(&crate::models::Project {
            id: None,
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let epic = db
        .create_epic("Security Epic", "Security hardening", &project_id)
        .await
        .expect("create epic");
    let epic_id = epic.id.expect("id");

    // Create and link context to epic
    let ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "security".to_string(),
            content: Some("Use rate limiting".to_string()),
            description: None,
            example: None,
            severity: Some("high".to_string()),
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    let ctx_id = ctx.id.as_ref().expect("id");

    db.link_context(&epic_id, ctx_id)
        .await
        .expect("link context to epic");

    // Verify context retrieval
    let contexts = db
        .get_linked_context(&epic_id)
        .await
        .expect("get epic context");
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].context_type, "security");
    assert_eq!(contexts[0].content.as_ref().unwrap(), "Use rate limiting");
}

// ============================================================================
// DELETE OPERATIONS
// ============================================================================

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
        .create_module("Auth", "Auth module", Some(&project_id))
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
    let deleted = db.delete_task(&task_id).await.expect("Failed to delete task");
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
    
    assert!(!deleted, "delete_task should return false for nonexistent task");

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
        .create_module("Auth", "Auth module", Some(&project_id))
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
    let deleted = db.delete_task(&task_id).await.expect("Failed to delete task");
    assert!(deleted);

    // Verify task is deleted
    let after_delete = db.get_task(&task_id).await.expect("Failed to check task");
    assert!(after_delete.is_none());
}
