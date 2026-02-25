//! Integration tests for SurrealDB backend.
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
