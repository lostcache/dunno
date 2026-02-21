use dunno::context::{get_file_context, get_subtask_context, get_task_context};
use dunno::db::DB;
use dunno::models::{Mistake, Project, SecurityDetail, StyleRule};

/// Sets up a fresh in-memory DB.
async fn setup_db() -> DB {
    DB::new("mem://").await.expect("Failed to init DB")
}

/// Helper: creates a full project → module → task hierarchy and links
/// knowledge nodes at each level. Returns (project_id, module_id, task_id).
async fn setup_hierarchy_with_context(db: &DB) -> (String, String, String) {
    let project = db
        .create_project(&Project {
            id: None,
            name: "TestProject".to_string(),
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

    // Link knowledge at each level
    let project_mistake = db
        .create_mistake(&Mistake {
            id: None,
            content: "Project Level Mistake".to_string(),
        })
        .await
        .expect("create mistake");
    db.link_context(&project_id, project_mistake.id.as_ref().unwrap())
        .await
        .expect("link project mistake");

    let module_style = db
        .create_style_rule(&StyleRule {
            id: None,
            description: "Module Level Style".to_string(),
            example: "example".to_string(),
        })
        .await
        .expect("create style rule");
    db.link_context(&module_id, module_style.id.as_ref().unwrap())
        .await
        .expect("link module style");

    let task_mistake = db
        .create_mistake(&Mistake {
            id: None,
            content: "Task Level Mistake".to_string(),
        })
        .await
        .expect("create task mistake");
    db.link_context(&task_id, task_mistake.id.as_ref().unwrap())
        .await
        .expect("link task mistake");

    (project_id, module_id, task_id)
}

#[tokio::test]
async fn test_task_hierarchy_context() {
    let db = setup_db().await;
    let (_project_id, _module_id, task_id) = setup_hierarchy_with_context(&db).await;

    let context = get_task_context(&task_id, &db)
        .await
        .expect("get_task_context should succeed");

    // Should contain knowledge from task, module, and project levels
    assert!(
        context.iter().any(|v| v["content"] == "Task Level Mistake"),
        "Missing task-level mistake. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Module Level Style"),
        "Missing module-level style. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Project Level Mistake"),
        "Missing project-level mistake. Context: {:?}",
        context
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

    // Link a mistake to the submodule level
    let sub_mistake = db
        .create_mistake(&Mistake {
            id: None,
            content: "Submodule Level Mistake".to_string(),
        })
        .await
        .expect("create sub mistake");
    db.link_context(&submodule_id, sub_mistake.id.as_ref().unwrap())
        .await
        .expect("link sub mistake");

    let context = get_file_context(&file_id, &db)
        .await
        .expect("get_file_context should succeed");

    // Should include context from submodule, module, and project
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Submodule Level Mistake"),
        "Missing submodule-level mistake. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Module Level Style"),
        "Missing module-level style. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Project Level Mistake"),
        "Missing project-level mistake. Context: {:?}",
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

    let context = get_file_context(&file_id, &db)
        .await
        .expect("get_file_context should succeed");

    // Should include context from module and project (no submodule)
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Module Level Style"),
        "Missing module-level style. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Project Level Mistake"),
        "Missing project-level mistake. Context: {:?}",
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

    // Link a mistake to the subtask level
    let st_mistake = db
        .create_mistake(&Mistake {
            id: None,
            content: "Subtask Level Mistake".to_string(),
        })
        .await
        .expect("create subtask mistake");
    db.link_context(&subtask_id, st_mistake.id.as_ref().unwrap())
        .await
        .expect("link subtask mistake");

    let context = get_subtask_context(&subtask_id, &db)
        .await
        .expect("get_subtask_context should succeed");

    // Should include context from subtask, task, module, and project (4 levels)
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Subtask Level Mistake"),
        "Missing subtask-level mistake. Context: {:?}",
        context
    );
    assert!(
        context.iter().any(|v| v["content"] == "Task Level Mistake"),
        "Missing task-level mistake. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Module Level Style"),
        "Missing module-level style. Context: {:?}",
        context
    );
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Project Level Mistake"),
        "Missing project-level mistake. Context: {:?}",
        context
    );
}

#[tokio::test]
async fn test_security_detail_in_context() {
    let db = setup_db().await;

    let project = db
        .create_project(&Project {
            id: None,
            name: "SecProject".to_string(),
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

    // Link a SecurityDetail to the module
    let detail = db
        .create_security_detail(&SecurityDetail {
            id: None,
            content: "SQL injection risk".to_string(),
            severity: "high".to_string(),
            category: "injection".to_string(),
            tags: vec!["sql".to_string()],
        })
        .await
        .expect("create security detail");
    db.link_context(&module_id, detail.id.as_ref().unwrap())
        .await
        .expect("link security detail");

    let context = get_task_context(&task_id, &db)
        .await
        .expect("get_task_context should succeed");

    assert!(
        context.iter().any(|v| v["content"] == "SQL injection risk"),
        "Missing security detail. Context: {:?}",
        context
    );
}
