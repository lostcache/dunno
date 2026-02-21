use lazydev::context::{get_file_context, get_task_context};
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::models::{File, Module, Project, Submodule, Task, TaskStatus};
use lazydev::vector_db::VectorDB;

#[tokio::test]
async fn test_hierarchy_context() -> anyhow::Result<()> {
    // 1. Setup DB (using kv-mem feature for in-memory)
    // Note: The main DB::new defaults to connecting to a server.
    // We'll instantiate a client manually here for the test if possible,
    // or just assume we can run against the local server if running tests.
    // Ideally we'd use a mock or in-memory DB.
    // Since `DB::new` connects to a URL, let's try connecting to a test namespace.

    let db = DB::new("mem://").await?;
    let vector_db = VectorDB::new("mem://").await?;

    // Create a unique project to isolate the test
    let project_name = format!("Test Project {}", uuid::Uuid::new_v4());

    // 2. Create Hierarchy
    let project = db
        .create_project(&Project {
            id: None,
            name: project_name,
            description: "Test Description".to_string(),
        })
        .await?;
    let project_id = project.id.unwrap();

    let module = db
        .create_module(&Module {
            id: None,
            project_id: project_id.clone(),
            name: "Test Module".to_string(),
            description: "Module Desc".to_string(),
        })
        .await?;
    let module_id = module.id.unwrap();

    let task = db
        .create_task(&Task {
            id: None,
            module_id: module_id.clone(),
            name: "Test Task".to_string(),
            description: "Task Desc".to_string(),
            status: TaskStatus::NotStarted,
        })
        .await?;
    let task_id = task.id.unwrap();

    // 3. Add Context at Different Levels

    // Task Level Mistake
    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Task Level Mistake".to_string(),
        Some(task_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    // Module Level Style Rule
    add_knowledge(
        "rust".to_string(),
        "style".to_string(),
        "Module Level Style".to_string(),
        Some(module_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    // Project Level Skill (or Mistake)
    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Project Level Mistake".to_string(),
        Some(project_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    // 4. Retrieve Context for Task
    let context = get_task_context(&task_id, &db, &vector_db).await?;

    // 5. Verify Results
    println!("Context Results: {:#?}", context);

    let has_task_mistake = context.iter().any(|v| v["content"] == "Task Level Mistake");
    let has_module_style = context
        .iter()
        .any(|v| v["description"] == "Module Level Style");
    let has_project_mistake = context
        .iter()
        .any(|v| v["content"] == "Project Level Mistake");

    assert!(has_task_mistake, "Missing Task Level Mistake");
    assert!(has_module_style, "Missing Module Level Style");
    assert!(has_project_mistake, "Missing Project Level Mistake");

    Ok(())
}

#[tokio::test]
async fn test_file_hierarchy_context() -> anyhow::Result<()> {
    let db = DB::new("mem://").await?;
    let vector_db = VectorDB::new("mem://").await?;

    let project_name = format!("Test File Project {}", uuid::Uuid::new_v4());
    let project = db
        .create_project(&Project {
            id: None,
            name: project_name,
            description: "Desc".to_string(),
        })
        .await?;
    let project_id = project.id.unwrap();

    let module = db
        .create_module(&Module {
            id: None,
            project_id: project_id.clone(),
            name: "Test Module".to_string(),
            description: "Desc".to_string(),
        })
        .await?;
    let module_id = module.id.unwrap();

    let submodule = db
        .create_submodule(&Submodule {
            id: None,
            module_id: module_id.clone(),
            name: "Test Submodule".to_string(),
            description: "Desc".to_string(),
        })
        .await?;
    let submodule_id = submodule.id.unwrap();

    let file = db
        .create_file(&File {
            id: None,
            module_id: module_id.clone(),
            submodule_id: Some(submodule_id.clone()),
            name: "test.rs".to_string(),
            path: "src/test.rs".to_string(),
        })
        .await?;
    let file_id = file.id.unwrap();

    // Add context to each layer
    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "File Level Mistake".to_string(),
        Some(file_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    add_knowledge(
        "rust".to_string(),
        "style".to_string(),
        "Submodule Level Style".to_string(),
        Some(submodule_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Module Level Mistake".to_string(),
        Some(module_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    add_knowledge(
        "rust".to_string(),
        "style".to_string(),
        "Project Level Style".to_string(),
        Some(project_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    let context = get_file_context(&file_id, &db, &vector_db).await?;
    println!("Context Results: {:#?}", context);

    assert!(context.iter().any(|v| v["content"] == "File Level Mistake"));
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Submodule Level Style")
    );
    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Module Level Mistake")
    );
    assert!(
        context
            .iter()
            .any(|v| v["description"] == "Project Level Style")
    );

    Ok(())
}

#[tokio::test]
async fn test_file_without_submodule_context() -> anyhow::Result<()> {
    let db = DB::new("mem://").await?;
    let vector_db = VectorDB::new("mem://").await?;

    let project = db
        .create_project(&Project {
            id: None,
            name: format!("NoSub Project {}", uuid::Uuid::new_v4()),
            description: "Desc".to_string(),
        })
        .await?;
    let project_id = project.id.unwrap();

    let module = db
        .create_module(&Module {
            id: None,
            project_id: project_id.clone(),
            name: "Mod".to_string(),
            description: "Desc".to_string(),
        })
        .await?;
    let module_id = module.id.unwrap();

    // File directly under module, no submodule
    let file = db
        .create_file(&File {
            id: None,
            module_id: module_id.clone(),
            submodule_id: None,
            name: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
        })
        .await?;
    let file_id = file.id.unwrap();

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Direct File Mistake".to_string(),
        Some(file_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    add_knowledge(
        "rust".to_string(),
        "style".to_string(),
        "Module Style".to_string(),
        Some(module_id.clone()),
        &db,
        &vector_db,
    )
    .await?;

    let context = get_file_context(&file_id, &db, &vector_db).await?;

    assert!(
        context
            .iter()
            .any(|v| v["content"] == "Direct File Mistake"),
        "Missing Direct File Mistake"
    );
    assert!(
        context.iter().any(|v| v["description"] == "Module Style"),
        "Missing Module Style"
    );

    Ok(())
}
