//! Integration tests for SurrealDB backend.
use super::*;
use crate::config::Config;
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
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let task = db
        .create_task("T", "d", Some(&module_id), Some(&project_id))
        .await
        .expect("create task");
    let task_id = task.id.expect("id");

    let submodule = db
        .create_module("SM", "d", &project_id, Some(module_id))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    let file = db
        .create_file(
            "f.rs",
            "src/f.rs",
            None,
            None,
            &project_id,
            Some(&submodule_id),
        )
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

    let task_ctx = crate::context::get_task_context(&task_id, false, &db)
        .await
        .expect("get_task_context");
    // Task context should only include context directly linked to the task, not from hierarchy
    assert!(
        task_ctx.contexts.is_empty(),
        "task context should only include directly linked context (task has none): {:?}",
        task_ctx
    );

    let file_ctx = crate::context::get_file_context(&file_id, false, &db)
        .await
        .expect("get_file_context");
    assert!(
        file_ctx.contexts.is_empty(),
        "file context should be file-only (no inherited submodule/project context): {:?}",
        file_ctx
    );
}

#[tokio::test]
async fn test_link_context_reverse_belongs_to() {
    let db = DB::new("mem://").await.expect("init DB");
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
        .create_module("M", "d", &project_id, None)
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

    let config = Config {
        backend: crate::config::StorageBackend::Embedded,
        local_path: db_path.to_string_lossy().to_string(),
        ..Config::default()
    };

    let db = DB::from_config(&config)
        .await
        .expect("local embedded config should surrealdb::engine::any::connect");
    let created = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Embedded".to_string(),
            description: "embedded local test".to_string(),
        })
        .await
        .expect("project create should work");
    assert!(!created.id.is_empty());

    let _ = cleanup_temp_db(db_path);
}

#[tokio::test]
async fn test_from_config_cloud_validation() {
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        ..Config::default()
    };
    let err = match DB::from_config(&config).await {
        Ok(_) => panic!("missing cloud fields should fail"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("url"));
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Testcrate::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
        .get_task_context(&task_id, false)
        .await
        .expect("get_task_context failed");

    assert_eq!(context.task.name, "Login");
    assert_eq!(context.contexts.len(), 1);
    assert_eq!(context.hierarchy.project_name, "Testcrate::models::Project");
    assert_eq!(context.hierarchy.module_name.as_deref(), Some("Auth"));
}

#[tokio::test]
async fn test_list_tasks_by_project() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Testcrate::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("crate::models::Module1", "First module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Testcrate::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("crate::models::Module1", "First module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Testcrate::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
        .get_task_context(&task_id, false)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Testcrate::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
        .get_task_context(&task_id, false)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let _submodule = db
        .create_module("JWT", "JWT submodule", &project_id, Some(module_id.clone()))
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
        .get_task_context(&task_id, false)
        .await
        .expect("get_task_context failed");
    assert_eq!(context.hierarchy.module_name.as_deref(), Some("Auth"));
}

#[tokio::test]
async fn test_get_task_context_files_from_module() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
        .get_task_context(&task_id, false)
        .await
        .expect("get_task_context failed");
    assert!(context.files.is_empty());
}

#[tokio::test]
async fn test_project_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test description".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let fetched = db.get_module(&module_id).await.expect("get_module failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Auth");
    let module_with_parent = db
        .create_module(
            "Auth2",
            "Auth2 module",
            &project_id,
            Some(module_id.clone()),
        )
        .await
        .expect("Failed to create module");
    let module_with_parent_id = module_with_parent.id.expect("module id");
    let fetched_with_parent = db
        .get_module(&module_with_parent_id)
        .await
        .expect("get_module failed");
    assert!(fetched_with_parent.is_some());
    assert_eq!(fetched_with_parent.as_ref().unwrap().name, "Auth2");
    assert_eq!(
        fetched_with_parent.unwrap().parent_module_id,
        Some(module_id)
    );
    let modules = db.list_modules().await.expect("list_modules failed");
    assert_eq!(modules.len(), 2);
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", project_id.as_ref(), None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let submodule = db
        .create_module(
            "JWT",
            "JWT submodule",
            project_id.as_ref(),
            Some(module_id.clone()),
        )
        .await
        .expect("Failed to create submodule");
    let submodule_id = submodule.id.expect("submodule id");
    let fetched = db
        .get_module(&submodule_id)
        .await
        .expect("get_submodule failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "JWT");
    let submodules = db.list_modules().await.expect("list_submodules failed");
    assert_eq!(submodules.len(), 2); // both the parent module and child module
    let submodules_by_module = db
        .list_modules_by_module(&module_id)
        .await
        .expect("list_submodules_by_module failed");
    assert_eq!(submodules_by_module.len(), 1);
}

#[tokio::test]
async fn test_file_operations() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", project_id.as_ref(), None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");
    let file = db
        .create_file(
            "main.rs",
            "src/main.rs",
            None,
            None,
            &project_id,
            Some(&module_id),
        )
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let todo = db
        .create_todo("Buy milk", Some(&project_id))
        .await
        .expect("Failed to create todo");
    let todo_id = todo.id.expect("todo id");
    let fetched = db.get_todo(&todo_id).await.expect("get_todo failed");
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.content, "Buy milk");
    assert_eq!(fetched.project_id, project_id);
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;
    let module = db
        .create_module("Auth", "Auth module", project_id.as_ref(), None)
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

// --- Module creation and link-after-create tests ---

#[tokio::test]
async fn test_create_module_with_project() {
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
        .create_module("M", "module with project", project_id.as_ref(), None)
        .await
        .expect("create module");
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
        by_project
            .iter()
            .any(|m| m.id.as_deref() == Some(module_id.as_str())),
        "module must appear under its project: {:?}",
        by_project
    );
    let all = db.list_modules().await.expect("list_modules");
    assert!(
        all.iter()
            .any(|m| m.id.as_deref() == Some(module_id.as_str())),
        "module must appear in list_modules"
    );
}

#[tokio::test]
async fn test_freestanding_file() {
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
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    let file = db
        .create_file("orphan.rs", "src/orphan.rs", None, None, &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "P".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

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
    let project1 = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "P1".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project1");
    let project1_id = project1.id;

    let project2 = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "P2".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project2");
    let project2_id = project2.id;

    let module = db
        .create_module("LaterLinked", "d", &project1_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    // Module is linked to project1 at creation; not yet linked to project2
    let by_p2_before = db
        .list_modules_by_project(&project2_id)
        .await
        .expect("list_modules_by_project p2");
    assert_eq!(by_p2_before.len(), 0);

    // Link module to project2 after creation
    db.link(&project2_id, "has_module", &module_id)
        .await
        .expect("link project2 -> has_module -> module");

    let by_p2_after = db
        .list_modules_by_project(&project2_id)
        .await
        .expect("list_modules_by_project p2 after");
    assert_eq!(by_p2_after.len(), 1);
    assert_eq!(by_p2_after[0].id.as_deref(), Some(module_id.as_str()));
}

#[tokio::test]
async fn test_link_after_create_task_hierarchy() {
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
        .create_module("M", "d", &project_id, None)
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
    assert_eq!(hierarchy.module_id.as_deref(), Some(module_id.as_str()));
}

#[tokio::test]
async fn test_create_with_link_ids_preserves_hierarchy() {
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
        .create_module("M", "d", &project_id, None)
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
    assert_eq!(hierarchy.module_id.as_deref(), Some(module_id.as_str()));
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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

    assert_eq!(task.status, crate::models::TaskStatus::Pending);

    let updated = db
        .update_task(
            &task_id,
            None,
            None,
            Some(crate::models::TaskStatus::Active),
        )
        .await
        .expect("Failed to update task status");

    assert!(updated.is_some());
    assert_eq!(updated.unwrap().status, crate::models::TaskStatus::Active);

    let finished = db
        .update_task(
            &task_id,
            None,
            None,
            Some(crate::models::TaskStatus::Completed),
        )
        .await
        .expect("Failed to update task to finished");

    assert_eq!(
        finished.unwrap().status,
        crate::models::TaskStatus::Completed
    );
}

#[tokio::test]
async fn test_update_task_all_fields() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            Some(crate::models::TaskStatus::Active),
        )
        .await
        .expect("Failed to update task");

    let task = updated.expect("Task should exist");
    assert_eq!(task.name, "New Name");
    assert_eq!(task.description, "New Description");
    assert_eq!(task.status, crate::models::TaskStatus::Active);
}

#[tokio::test]
async fn test_update_task_empty_patch_returns_current() {
    let db = DB::new("mem://").await.expect("Failed to init DB");
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let submodule = db
        .create_module("OAuth", "OAuth submodule", &project_id, Some(module_id))
        .await
        .expect("Failed to create submodule");
    let submodule_id = submodule.id.expect("submodule id");

    let file1 = db
        .create_file(
            "oauth.rs",
            "src/auth/oauth.rs",
            None,
            None,
            &project_id,
            Some(&submodule_id),
        )
        .await
        .expect("Failed to create file 1");
    let file1_id = file1.id.expect("file id");

    let file2 = db
        .create_file(
            "jwt.rs",
            "src/auth/jwt.rs",
            None,
            None,
            &project_id,
            Some(&submodule_id),
        )
        .await
        .expect("Failed to create file 2");
    let _file2_id = file2.id.expect("file id");

    let files_by_submodule = db
        .list_files_by_module(&submodule_id)
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
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        url: "wss://test.surrealdb.com".to_string(),
        namespace: "".to_string(),
        database: "test".to_string(),
        ..Config::default()
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing namespace should fail");
    assert!(err.to_string().contains("namespace"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_database() {
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        url: "wss://test.surrealdb.com".to_string(),
        namespace: "test".to_string(),
        database: "".to_string(),
        ..Config::default()
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing database should fail");
    assert!(err.to_string().contains("database"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_username() {
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        url: "wss://test.surrealdb.com".to_string(),
        namespace: "test".to_string(),
        database: "test".to_string(),
        username: "".to_string(),
        ..Config::default()
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing username should fail");
    assert!(err.to_string().contains("username"));
}

#[tokio::test]
async fn test_from_config_cloud_missing_password() {
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        url: "wss://test.surrealdb.com".to_string(),
        namespace: "test".to_string(),
        database: "test".to_string(),
        password: "".to_string(),
        ..Config::default()
    };

    let err = DB::from_config(&config)
        .await
        .expect_err("missing password should fail");
    assert!(err.to_string().contains("password"));
}

#[tokio::test]
async fn test_from_config_cloud_valid() {
    let config = Config {
        backend: crate::config::StorageBackend::Cloud,
        url: "wss://test.surrealdb.com".to_string(),
        namespace: "test".to_string(),
        database: "test".to_string(),
        ..Config::default()
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
        .get_task_context("task:nonexistent", false)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "For user story testing".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

    let module = db
        .create_module("Core", "Core module", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

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

    // Verify module link
    let modules = db
        .list_modules_by_user_story(&us_id)
        .await
        .expect("list modules by user story");
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "Core");

    // Verify reverse lookup
    let stories_from_module = db
        .list_user_stories_by_module(&module_id)
        .await
        .expect("list user stories by module");
    assert_eq!(stories_from_module.len(), 1);
}

#[tokio::test]
async fn test_epic_crud() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create a project
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test Project".to_string(),
            description: "For epic testing".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
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
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "Test".to_string(),
            description: "d".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id;

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

#[tokio::test]
async fn test_get_task_context_with_files_and_linked_context() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
    let project = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "TestProject".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("Failed to create project");
    let project_id = project.id;

    // Create module with context (this should NOT appear in task context)
    let module = db
        .create_module("Auth", "Auth module", &project_id, None)
        .await
        .expect("Failed to create module");
    let module_id = module.id.expect("module id");

    let module_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "style".to_string(),
            content: Some("Module level style".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create module context");
    db.link_context(&module_id, &module_ctx.id.unwrap())
        .await
        .expect("link module context");

    // Create file under module with context (this should NOT appear in task context)
    let file = db
        .create_file(
            "auth.rs",
            "src/auth.rs",
            Some("Auth implementation"),
            None,
            &project_id,
            Some(&module_id),
        )
        .await
        .expect("Failed to create file");
    let file_id = file.id.expect("file id");

    let file_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("File level mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create file context");
    db.link_context(&file_id, &file_ctx.id.unwrap())
        .await
        .expect("link file context");

    // Create task
    let task = db
        .create_task(
            "Implement Auth",
            "Add authentication",
            Some(&module_id),
            Some(&project_id),
        )
        .await
        .expect("Failed to create task");
    let task_id = task.id.expect("task id");

    // Link context directly to task (this SHOULD appear in task context)
    let task_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            context_type: "security".to_string(),
            content: Some("Task level security".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("Failed to create task context");
    db.link_context(&task_id, &task_ctx.id.unwrap())
        .await
        .expect("link task context");

    // Verify file was created and linked to module
    let files_in_module = db
        .list_files_by_module(&module_id)
        .await
        .expect("list files");
    assert_eq!(files_in_module.len(), 1, "Should have 1 file in module");

    // Link a different file directly to the task
    let task_file = db
        .create_file(
            "task.rs",
            "src/task.rs",
            Some("Task-specific file"),
            None,
            &project_id,
            None,
        )
        .await
        .expect("create task file");
    let task_file_id = task_file.id.as_ref().unwrap().clone();
    db.link(&task_file_id, "belongs_to_task", &task_id)
        .await
        .expect("link file to task");

    // Get task context
    let context = db
        .get_task_context(&task_id, false)
        .await
        .expect("get_task_context failed");

    // Should have ONLY the context directly linked to the task (1 context)
    assert_eq!(
        context.contexts.len(),
        1,
        "Expected only 1 context directly linked to task"
    );
    assert_eq!(
        context.contexts[0].content,
        Some("Task level security".to_string())
    );

    // Should NOT include module or file context
    assert!(
        !context
            .contexts
            .iter()
            .any(|c| c.content.as_deref() == Some("Module level style")),
        "Should NOT include module context"
    );
    assert!(
        !context
            .contexts
            .iter()
            .any(|c| c.content.as_deref() == Some("File level mistake")),
        "Should NOT include file context"
    );

    // Files should be the ones directly linked to the task, not module files
    assert_eq!(context.files.len(), 1);
    assert_eq!(context.files[0].id.as_deref().unwrap(), task_file_id);
    assert_eq!(context.files[0].name, "task.rs");
    // The module file must NOT appear
    assert!(
        !context
            .files
            .iter()
            .any(|f| f.id.as_deref() == Some(file_id.as_str()))
    );

    // Verify hierarchy is correct
    assert_eq!(context.hierarchy.project_id, project_id);
    assert_eq!(
        context.hierarchy.module_id.as_deref(),
        Some(module_id.as_str())
    );
}

#[tokio::test]
async fn test_list_submodules_by_project() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Create submodule linked to module
    let submodule = db
        .create_module("S", "d", &project_id, Some(module_id.clone()))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    // List all modules (parent + child)
    let all = db.list_modules().await.expect("list_submodules");
    assert_eq!(all.len(), 2);

    // List modules by project returns only top-level modules (M, not S)
    let by_project = db
        .list_modules_by_project(&project_id)
        .await
        .expect("list_submodules_by_project");
    assert_eq!(by_project.len(), 1);
    assert_eq!(by_project[0].id, Some(module_id.clone()));

    // List submodules by module (should find the submodule)
    let by_module = db
        .list_modules_by_module(&module_id)
        .await
        .expect("list_submodules_by_module");
    assert_eq!(by_module.len(), 1);
    assert_eq!(by_module[0].id, Some(submodule_id.clone()));
}

#[tokio::test]
async fn test_list_files_by_project() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Create submodule linked to module
    let submodule = db
        .create_module("S", "d", &project_id, Some(module_id.clone()))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    // Create file linked to module
    let file1 = db
        .create_file(
            "f1.rs",
            "src/f1.rs",
            Some("d"),
            None,
            &project_id,
            Some(&module_id),
        )
        .await
        .expect("create file 1");
    let file1_id = file1.id.expect("id");

    // Create file linked to submodule
    let file2 = db
        .create_file(
            "f2.rs",
            "src/f2.rs",
            Some("d"),
            None,
            &project_id,
            Some(&submodule_id),
        )
        .await
        .expect("create file 2");
    let file2_id = file2.id.expect("id");

    // List all files (should include both)
    let all = db.list_files().await.expect("list_files");
    assert_eq!(all.len(), 2);

    // List files by project (should find both files)
    let by_project = db
        .list_files_by_project(&project_id)
        .await
        .expect("list_files_by_project");
    assert_eq!(by_project.len(), 2);

    // List files by module (should find file1)
    let by_module = db
        .list_files_by_module(&module_id)
        .await
        .expect("list_files_by_module");
    assert_eq!(by_module.len(), 1);
    assert_eq!(by_module[0].id, Some(file1_id.clone()));

    // List files by submodule (should find file2)
    let by_submodule = db
        .list_files_by_module(&submodule_id)
        .await
        .expect("list_files_by_submodule");
    assert_eq!(by_submodule.len(), 1);
    assert_eq!(by_submodule[0].id, Some(file2_id.clone()));
}

#[tokio::test]
async fn test_module_belongs_to_project_edge() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Verify module can be found via belongs_to_project edge by querying the graph
    let mut response = db
        .client
        .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($mid)")
        .bind(("mid", module_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_project_id = json
        .get("pid")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("project id");
    assert_eq!(found_project_id, project_id);
}

#[tokio::test]
async fn test_submodule_belongs_to_edges() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Create submodule linked to module
    let submodule = db
        .create_module("S", "d", &project_id, Some(module_id.clone()))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    // Verify submodule has belongs_to_module edge
    let mut response = db
        .client
        .query("SELECT ->belongs_to_module->module.id AS mid FROM ONLY type::record($sid)")
        .bind(("sid", submodule_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_module_id = json
        .get("mid")
        .and_then(|m| m.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("module id");
    assert_eq!(found_module_id, module_id);

    // Verify submodule has belongs_to_project edge
    let mut response = db
        .client
        .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($sid)")
        .bind(("sid", submodule_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_project_id = json
        .get("pid")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("project id");
    assert_eq!(found_project_id, project_id);
}

#[tokio::test]
async fn test_file_belongs_to_edges() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Create file linked to module
    let file = db
        .create_file(
            "test.rs",
            "src/test.rs",
            Some("d"),
            None,
            &project_id,
            Some(&module_id),
        )
        .await
        .expect("create file");
    let file_id = file.id.expect("id");

    // Verify file has belongs_to_module edge
    let mut response = db
        .client
        .query("SELECT ->belongs_to_module->module.id AS mid FROM ONLY type::record($fid)")
        .bind(("fid", file_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_module_id = json
        .get("mid")
        .and_then(|m| m.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("module id");
    assert_eq!(found_module_id, module_id);

    // Verify file has belongs_to_project edge
    let mut response = db
        .client
        .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($fid)")
        .bind(("fid", file_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_project_id = json
        .get("pid")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("project id");
    assert_eq!(found_project_id, project_id);
}

#[tokio::test]
async fn test_file_belongs_to_edges_with_submodule() {
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Create project
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

    // Create module linked to project
    let module = db
        .create_module("M", "d", &project_id, None)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    // Create submodule linked to module
    let submodule = db
        .create_module("S", "d", &project_id, Some(module_id.clone()))
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("id");

    // Create file linked to submodule
    let file = db
        .create_file(
            "test.rs",
            "src/test.rs",
            Some("d"),
            None,
            &project_id,
            Some(&submodule_id),
        )
        .await
        .expect("create file");
    let file_id = file.id.expect("id");

    // Verify file has belongs_to_module edge (direct link to child module)
    let mut response = db
        .client
        .query("SELECT ->belongs_to_module->module.id AS mid FROM ONLY type::record($fid)")
        .bind(("fid", file_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_module_id = json
        .get("mid")
        .and_then(|m| m.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("module id");
    assert_eq!(found_module_id, submodule_id); // file links directly to child module

    // Verify file has belongs_to_project edge (cascaded from submodule)
    let mut response = db
        .client
        .query("SELECT ->belongs_to_project->project.id AS pid FROM ONLY type::record($fid)")
        .bind(("fid", file_id.clone()))
        .await
        .expect("query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_project_id = json
        .get("pid")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("project id");
    assert_eq!(found_project_id, project_id);
}

#[tokio::test]
async fn test_context_inheritance_project() {
    let db = DB::new("mem://").await.expect("init db");
    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;

    let ctx1 = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("c1".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    db.link_context(&pid, ctx1.id.as_ref().unwrap())
        .await
        .unwrap();

    let contexts = db.get_project_context(&pid, true).await.unwrap();
    assert_eq!(contexts.len(), 1);
    let contents: Vec<String> = contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(contents.contains(&"c1".into()));
}

#[tokio::test]
async fn test_context_inheritance_module() {
    let db = DB::new("mem://").await.expect("init db");
    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let m = db.create_module("m", "d", &pid, None).await.unwrap();
    let mid = m.id.unwrap();

    let p_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("p_rule".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let m_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("m_rule".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    db.link_context(&pid, p_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&mid, m_ctx.id.as_ref().unwrap())
        .await
        .unwrap();

    let contexts = db.get_module_context(&mid, true).await.unwrap();

    assert_eq!(contexts.len(), 2);
    let contents: Vec<String> = contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(contents.contains(&"p_rule".to_string()));
    assert!(contents.contains(&"m_rule".to_string()));
}

#[tokio::test]
async fn test_context_inheritance_submodule() {
    let db = DB::new("mem://").await.expect("init db");
    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let m = db.create_module("m", "d", &pid, None).await.unwrap();
    let mid = m.id.unwrap();
    let s = db
        .create_module("s", "d", &pid, Some(mid.clone()))
        .await
        .unwrap();
    let sid = s.id.unwrap();

    let p_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("p".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let m_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("m".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let s_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("s".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    db.link_context(&pid, p_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&mid, m_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&sid, s_ctx.id.as_ref().unwrap())
        .await
        .unwrap();

    let contexts = db.get_module_context(&sid, true).await.unwrap();

    assert_eq!(contexts.len(), 3);
    let contents: Vec<String> = contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(contents.contains(&"p".to_string()));
    assert!(contents.contains(&"m".to_string()));
    assert!(contents.contains(&"s".to_string()));
}

#[tokio::test]
async fn test_context_inheritance_task() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let m = db.create_module("m", "d", &pid, None).await.unwrap();
    let mid = m.id.unwrap();
    let s = db
        .create_module("s", "d", &pid, Some(mid.clone()))
        .await
        .unwrap();
    let sid = s.id.unwrap();
    let t = db
        .create_task("t", "d", Some(&sid), Some(&pid))
        .await
        .unwrap();
    let tid = t.id.unwrap();

    let p_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("p".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let m_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("m".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let s_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("s".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let t_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("t".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    db.link_context(&pid, p_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&mid, m_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&sid, s_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&tid, t_ctx.id.as_ref().unwrap())
        .await
        .unwrap();

    let ctx_full = db
        .get_task_context(&tid, true)
        .await
        .expect("get full failed");

    assert_eq!(
        ctx_full.contexts.len(),
        4,
        "Should have exactly 4 inherited contexts"
    );
    let contents: Vec<String> = ctx_full
        .contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(
        contents.contains(&"p".to_string()),
        "Missing project context"
    );
    assert!(
        contents.contains(&"m".to_string()),
        "Missing module context"
    );
    assert!(
        contents.contains(&"s".to_string()),
        "Missing submodule context"
    );
    assert!(contents.contains(&"t".to_string()), "Missing task context");
}

#[tokio::test]
async fn test_context_inheritance_file() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let m = db.create_module("m", "d", &pid, None).await.unwrap();
    let mid = m.id.unwrap();
    let f = db
        .create_file("f", "src/f.rs", None, None, &pid, Some(&mid))
        .await
        .unwrap();
    let fid = f.id.unwrap();

    let p_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("p".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let m_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("m".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let f_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("f".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    db.link_context(&pid, p_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&mid, m_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&fid, f_ctx.id.as_ref().unwrap())
        .await
        .unwrap();

    let ctx = db
        .get_file_context(&fid, true)
        .await
        .expect("get full context");
    assert_eq!(ctx.contexts.len(), 3);
    let contents: Vec<String> = ctx
        .contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(contents.contains(&"p".into()));
    assert!(contents.contains(&"m".into()));
    assert!(contents.contains(&"f".into()));
}

#[tokio::test]
async fn test_context_inheritance_epic() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "p".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let e = db.create_epic("e", "d", &pid).await.unwrap();
    let eid = e.id.unwrap();

    let p_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("p".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let e_ctx = db
        .create_context(&crate::models::Context {
            id: None,
            content: Some("e".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    db.link_context(&pid, p_ctx.id.as_ref().unwrap())
        .await
        .unwrap();
    db.link_context(&eid, e_ctx.id.as_ref().unwrap())
        .await
        .unwrap();

    let ctx = db
        .get_epic_context(&eid, true)
        .await
        .expect("get full context");
    assert_eq!(ctx.contexts.len(), 2);
    let contents: Vec<String> = ctx
        .contexts
        .iter()
        .map(|c| c.content.clone().unwrap())
        .collect();
    assert!(contents.contains(&"p".into()));
    assert!(contents.contains(&"e".into()));
}

#[tokio::test]
async fn test_task_ctx_full_includes_persona_workflow() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "proj".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let m = db.create_module("mod", "d", &pid, None).await.unwrap();
    let mid = m.id.unwrap();
    let t = db
        .create_task("task", "d", Some(&mid), Some(&pid))
        .await
        .unwrap();
    let tid = t.id.unwrap();

    db.create_persona("P1", "persona content", &pid)
        .await
        .unwrap();
    db.create_workflow("W1", "workflow content", &pid)
        .await
        .unwrap();

    // Node mode: persona and workflow should be empty
    let node_ctx = db.get_task_context(&tid, false).await.expect("node ctx");
    assert!(node_ctx.persona.is_empty());
    assert!(node_ctx.workflow.is_empty());

    // Full mode: persona and workflow should be populated
    let full_ctx = db.get_task_context(&tid, true).await.expect("full ctx");
    assert_eq!(full_ctx.persona.len(), 1);
    assert_eq!(full_ctx.persona[0].name, "P1");
    assert_eq!(full_ctx.workflow.len(), 1);
    assert_eq!(full_ctx.workflow[0].name, "W1");

    // Isolation: another project's persona must not appear
    let p2 = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "other".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    db.create_persona("P2", "other persona", &p2.id)
        .await
        .unwrap();
    let full_ctx2 = db.get_task_context(&tid, true).await.expect("full ctx2");
    assert_eq!(full_ctx2.persona.len(), 1);
}

#[tokio::test]
async fn test_file_ctx_full_includes_persona_workflow() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "proj".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let f = db
        .create_file("f", "src/f.rs", None, None, &pid, None)
        .await
        .unwrap();
    let fid = f.id.unwrap();

    db.create_persona("P1", "persona content", &pid)
        .await
        .unwrap();
    db.create_workflow("W1", "workflow content", &pid)
        .await
        .unwrap();

    // Node mode: persona and workflow should be empty
    let node_ctx = db.get_file_context(&fid, false).await.expect("node ctx");
    assert!(node_ctx.persona.is_empty());
    assert!(node_ctx.workflow.is_empty());

    // Full mode: persona and workflow should be populated
    let full_ctx = db.get_file_context(&fid, true).await.expect("full ctx");
    assert_eq!(full_ctx.persona.len(), 1);
    assert_eq!(full_ctx.persona[0].name, "P1");
    assert_eq!(full_ctx.workflow.len(), 1);
    assert_eq!(full_ctx.workflow[0].name, "W1");
}

#[tokio::test]
async fn test_epic_ctx_full_includes_persona_workflow() {
    let db = DB::new("mem://").await.expect("init db");

    let p = db
        .create_project(&crate::models::Project {
            // Empty string is skipped during serialization so SurrealDB auto-generates the record ID.
            // Use `String::new()` as a placeholder when creating a new project.
            id: String::new(),
            name: "proj".into(),
            description: "d".into(),
        })
        .await
        .unwrap();
    let pid = p.id;
    let e = db.create_epic("epic", "d", &pid).await.unwrap();
    let eid = e.id.unwrap();

    db.create_persona("P1", "persona content", &pid)
        .await
        .unwrap();
    db.create_workflow("W1", "workflow content", &pid)
        .await
        .unwrap();

    // Node mode: persona and workflow should be empty
    let node_ctx = db.get_epic_context(&eid, false).await.expect("node ctx");
    assert!(node_ctx.persona.is_empty());
    assert!(node_ctx.workflow.is_empty());

    // Full mode: persona and workflow should be populated
    let full_ctx = db.get_epic_context(&eid, true).await.expect("full ctx");
    assert_eq!(full_ctx.persona.len(), 1);
    assert_eq!(full_ctx.persona[0].name, "P1");
    assert_eq!(full_ctx.workflow.len(), 1);
    assert_eq!(full_ctx.workflow[0].name, "W1");
}

#[tokio::test]
async fn test_child_module_belongs_to_module_edge_and_graph() {
    let db = DB::new("mem://").await.expect("init db");

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
        .create_module("Parent", "d", &project_id, None)
        .await
        .expect("create parent module");
    let parent_id = parent.id.expect("parent id");

    let child = db
        .create_module("Child", "d", &project_id, Some(parent_id.clone()))
        .await
        .expect("create child module");
    let child_id = child.id.expect("child id");

    // Verify the belongs_to_module edge from child -> parent exists in the DB
    let mut response = db
        .client
        .query("SELECT ->belongs_to_module->module.id AS mid FROM ONLY type::record($cid)")
        .bind(("cid", child_id.clone()))
        .await
        .expect("raw query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_parent_id = json
        .get("mid")
        .and_then(|m| m.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("parent module id from edge");
    assert_eq!(found_parent_id, parent_id);

    // Verify get_graph_data_by_project returns the belongs_to_module edge
    let graph = db
        .get_graph_data_by_project(&project_id)
        .await
        .expect("get graph data");
    let elements = graph["elements"].as_array().expect("elements array");
    let has_btm_edge = elements.iter().any(|el| {
        el["data"]["edge_type"].as_str() == Some("belongs_to_module")
            && el["data"]["source"].as_str() == Some(&child_id)
            && el["data"]["target"].as_str() == Some(&parent_id)
    });
    assert!(
        has_btm_edge,
        "belongs_to_module edge from child to parent not found in graph data"
    );
}

#[tokio::test]
async fn test_child_module_has_module_edge_and_graph() {
    let db = DB::new("mem://").await.expect("init db");

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
        .create_module("Parent", "d", &project_id, None)
        .await
        .expect("create parent module");
    let parent_id = parent.id.expect("parent id");

    let child = db
        .create_module("Child", "d", &project_id, Some(parent_id.clone()))
        .await
        .expect("create child module");
    let child_id = child.id.expect("child id");

    // Verify has_module edge: parent -> child
    let mut response = db
        .client
        .query("SELECT ->has_module->module.id AS mid FROM ONLY type::record($pid)")
        .bind(("pid", parent_id.clone()))
        .await
        .expect("raw query");
    let record: Option<surrealdb::types::Value> = response.take(0).expect("take");
    let json = crate::db::surreal::util::surreal_to_json(record.expect("record"));
    let found_child_id = json
        .get("mid")
        .and_then(|m| m.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .expect("child module id from has_module edge");
    assert_eq!(found_child_id, child_id);

    // Verify get_graph_data_by_project returns the has_module edge
    let graph = db
        .get_graph_data_by_project(&project_id)
        .await
        .expect("get graph data");
    let elements = graph["elements"].as_array().expect("elements array");
    let has_hm_edge = elements.iter().any(|el| {
        el["data"]["edge_type"].as_str() == Some("has_module")
            && el["data"]["source"].as_str() == Some(&parent_id)
            && el["data"]["target"].as_str() == Some(&child_id)
    });
    assert!(
        has_hm_edge,
        "has_module edge from parent to child not found in graph data"
    );
}
