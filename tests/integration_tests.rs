/// Sets up a fresh in-memory dunno::db::DB.
async fn setup_db() -> dunno::db::DB {
    dunno::db::DB::new("mem://").await.expect("Failed to init dunno::db::DB")
}

/// Helper: creates a full project → module → task hierarchy and links
/// knowledge nodes at each level. Returns (project_id, module_id, task_id).
async fn setup_hierarchy_with_context(db: &dunno::db::DB) -> (String, String, String) {
    let project = db
        .create_project(&dunno::models::Project {
            id: None,
            name: "Testdunno::models::Project".to_string(),
            description: "A test project".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("project id");

    let module = db
        .create_module("Auth", "Auth module", &project_id)
        .await
        .expect("create module");
    let module_id = module.id.expect("module id");

    let task = db
        .create_task("Login", "Implement login", &module_id, &project_id)
        .await
        .expect("create task");
    let task_id = task.id.expect("task id");

    // Link context at project/module/task levels (hierarchy behavior is now encoded in tests, but retrieval is task-only).
    let project_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("dunno::models::Project Level dunno::models::Mistake".to_string()),
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

    let module_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "style_rule".to_string(),
            content: None,
            description: Some("Module Level Style".to_string()),
            example: Some("example".to_string()),
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    db.link_context(&module_id, module_ctx.id.as_ref().unwrap())
        .await
        .expect("link module context");

    let task_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Task Level dunno::models::Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create task context");
    db.link_context(&task_id, task_ctx.id.as_ref().unwrap())
        .await
        .expect("link task context");

    (project_id, module_id, task_id)
}

#[tokio::test]
async fn test_task_hierarchy_context() {
    let db = setup_db().await;
    let (_project_id, _module_id, task_id) = setup_hierarchy_with_context(&db).await;

    let context = dunno::context::get_task_context(&task_id, &db)
        .await
        .expect("dunno::context::get_task_context should succeed");

    // With direct-only retrieval, task context should only contain the task-level entry.
    assert!(
        context.iter().any(|v| v["content"] == "Task Level dunno::models::Mistake"),
        "Missing task-level context. Context: {:?}",
        context
    );
    assert_eq!(context.len(), 1, "Context should be task-only: {:?}", context);
}

#[tokio::test]
async fn test_belongs_to_reverse_edges() {
    let db = setup_db().await;
    let project = db
        .create_project(&dunno::models::Project {
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

    let project_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("project mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    let project_ctx_id = project_ctx.id.as_ref().unwrap().clone();
    db.link_context(&project_id, &project_ctx_id)
        .await
        .expect("link project context");

    let task_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("task mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create context");
    let task_ctx_id = task_ctx.id.as_ref().unwrap().clone();
    db.link_context(&task_id, &task_ctx_id)
        .await
        .expect("link task context");

    let project_targets = db
        .get_belongs_to_targets(&project_ctx_id)
        .await
        .expect("get_belongs_to_targets");
    assert!(
        project_targets.contains(&project_id),
        "project-linked context should belong_to project: {:?}",
        project_targets
    );

    let task_targets = db
        .get_belongs_to_targets(&task_ctx_id)
        .await
        .expect("get_belongs_to_targets");
    assert!(
        task_targets.contains(&project_id),
        "task-linked context should belong_to project: {:?}",
        task_targets
    );
    assert!(
        task_targets.contains(&module_id),
        "task-linked context should belong_to module: {:?}",
        task_targets
    );
    assert!(
        task_targets.contains(&task_id),
        "task-linked context should belong_to task: {:?}",
        task_targets
    );
}

#[tokio::test]
async fn test_file_hierarchy_context() {
    let db = setup_db().await;
    let (_project_id, module_id, _task_id) = setup_hierarchy_with_context(&db).await;

    let submodule = db
        .create_submodule("Controllers", "Controllers submodule", &module_id)
        .await
        .expect("create submodule");
    let submodule_id = submodule.id.expect("submodule id");

    let file = db
        .create_file(
            "auth_controller.rs",
            "src/controllers/auth.rs",
            &submodule_id,
        )
        .await
        .expect("create file");
    let file_id = file.id.expect("file id");

    // Link context to the submodule level
    let sub_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Submodule Level dunno::models::Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create sub context");
    db.link_context(&submodule_id, sub_ctx.id.as_ref().unwrap())
        .await
        .expect("link sub context");

    let context = dunno::context::get_file_context(&file_id, &db)
        .await
        .expect("dunno::context::get_file_context should succeed");

    // With direct-only retrieval, file context should only contain the file-level entries (none in this setup),
    // so inherited submodule/module/project context should not appear here.
    assert!(
        !context
            .iter()
            .any(|v| v["content"] == "Submodule Level dunno::models::Mistake"),
        "file context should not inherit submodule context: {:?}",
        context
    );
}

#[tokio::test]
async fn test_file_without_submodule_context() {
    let db = setup_db().await;
    let (_project_id, module_id, _task_id) = setup_hierarchy_with_context(&db).await;

    // File directly under module (no submodule)
    let file = db
        .create_file("lib.rs", "src/lib.rs", &module_id)
        .await
        .expect("create file");
    let file_id = file.id.expect("file id");

    let context = dunno::context::get_file_context(&file_id, &db)
        .await
        .expect("dunno::context::get_file_context should succeed");

    // With direct-only retrieval, this file has no directly linked context, even though its module/project do.
    assert!(
        context.is_empty(),
        "file context should be direct-only and empty here: {:?}",
        context
    );
}

#[tokio::test]
async fn test_subtask_four_level_context() {
    let db = setup_db().await;
    let (_project_id, _module_id, task_id) = setup_hierarchy_with_context(&db).await;

    let subtask = db
        .create_subtask("Write Tests", "Unit tests for login", &task_id)
        .await
        .expect("create subtask");
    let subtask_id = subtask.id.expect("subtask id");

    // Link context to the subtask level
    let st_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "mistake".to_string(),
            content: Some("Subtask Level dunno::models::Mistake".to_string()),
            description: None,
            example: None,
            severity: None,
            category: None,
            tags: None,
        })
        .await
        .expect("create subtask context");
    db.link_context(&subtask_id, st_ctx.id.as_ref().unwrap())
        .await
        .expect("link subtask context");

    let context = dunno::context::get_subtask_context(&subtask_id, &db)
        .await
        .expect("dunno::context::get_subtask_context should succeed");

    // With direct-only retrieval, subtask context should only contain the subtask-level entry.
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Subtask Level dunno::models::Mistake"),
        "Missing subtask-level context. Context: {:?}",
        context
    );
    assert!(
        context.len() == 1,
        "Subtask context should be direct-only: {:?}",
        context
    );
}

#[tokio::test]
async fn test_security_detail_in_context() {
    let db = setup_db().await;

    let project = db
        .create_project(&dunno::models::Project {
            id: None,
            name: "Secdunno::models::Project".to_string(),
            description: "Test".to_string(),
        })
        .await
        .expect("create project");
    let project_id = project.id.expect("id");

    let module = db
        .create_module("API", "API module", &project_id)
        .await
        .expect("create module");
    let module_id = module.id.expect("id");

    let task = db
        .create_task("Fix SQL", "Fix injection", &module_id, &project_id)
        .await
        .expect("create task");
    let task_id = task.id.expect("id");

    // Link a security_detail context to the task (since retrieval is direct-only).
    let detail_ctx = db
        .create_context(&dunno::models::Context {
            id: None,
            context_type: "security_detail".to_string(),
            content: Some("SQL injection risk".to_string()),
            description: None,
            example: None,
            severity: Some("high".to_string()),
            category: Some("injection".to_string()),
            tags: Some(vec!["sql".to_string()]),
        })
        .await
        .expect("create context");
    db.link_context(&task_id, detail_ctx.id.as_ref().unwrap())
        .await
        .expect("link security context");

    let context = dunno::context::get_task_context(&task_id, &db)
        .await
        .expect("dunno::context::get_task_context should succeed");

    assert!(
        context.iter().any(|v| v["content"] == "SQL injection risk"),
        "Missing security detail. Context: {:?}",
        context
    );
}
