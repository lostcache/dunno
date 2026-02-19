use lazydev::context::get_context;
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::vector_db::VectorDB;

#[tokio::test]
async fn test_add_then_context_flow() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");
    vector_db
        .ensure_collection("knowledge", 384)
        .await
        .expect("Failed to ensure collection");

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Avoid unwrap in production code".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add knowledge");

    let results = get_context("avoid unwrap".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(!results.is_empty(), "Expected at least one context result");
    assert!(
        results
            .iter()
            .any(|item| item["content"] == "Avoid unwrap in production code"),
        "Expected the inserted mistake in retrieved context"
    );
    assert!(
        results.iter().all(|item| item.get("node_type").is_some()),
        "Expected each context result to include node_type"
    );
}

#[tokio::test]
async fn test_context_traverses_shared_tag_graph() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Using unwrap without context".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add mistake");

    add_knowledge(
        "rust".to_string(),
        "style".to_string(),
        "Prefer explicit error handling with anyhow".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add style rule");

    let results = get_context("rust error handling".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(
        results.iter().any(|item| item["node_type"] == "mistake"),
        "Expected a mistake node in graph results"
    );
    assert!(
        results.iter().any(|item| item["node_type"] == "style_rule"),
        "Expected a style_rule node in graph results"
    );
}

#[tokio::test]
async fn test_context_includes_skill_nodes() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");

    add_knowledge(
        "backend".to_string(),
        "skill".to_string(),
        "Design resilient APIs".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add skill");

    let results = get_context("resilient api design".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(
        results.iter().any(|item| item["node_type"] == "skill"),
        "Expected a skill node in graph results"
    );
    assert!(
        results
            .iter()
            .any(|item| item["name"] == "Design resilient APIs"),
        "Expected the inserted skill in retrieved context"
    );
}

#[tokio::test]
async fn test_context_returns_empty_for_unrelated_query() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Avoid panicking in library code".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add mistake");

    let results = get_context("javascript dom rendering".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(
        results.is_empty(),
        "Expected no results for an unrelated query"
    );
}

#[tokio::test]
async fn test_context_query_is_case_insensitive_for_category_seed() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");

    add_knowledge(
        "Rust Lang".to_string(),
        "mistake".to_string(),
        "Do not ignore Result values".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add mistake");

    let results = get_context("RUST".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(
        results
            .iter()
            .any(|item| item["content"] == "Do not ignore Result values"),
        "Expected category match to seed graph traversal"
    );
}

#[tokio::test]
async fn test_context_traversal_respects_hop_limit() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://")
        .await
        .expect("Failed to init VectorDB");

    add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Avoid unwrap for recoverable errors".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add mistake");

    add_knowledge(
        "backend".to_string(),
        "skill".to_string(),
        "Implement retry policies".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add near skill");

    add_knowledge(
        "distributed".to_string(),
        "style".to_string(),
        "Use idempotent handlers".to_string(),
        &db,
        &vector_db,
    )
    .await
    .expect("Failed to add deep style rule");

    let rust_tag = db
        .create_or_get_category_tag("rust")
        .await
        .expect("Failed to create rust tag");
    let backend_tag = db
        .create_or_get_category_tag("backend")
        .await
        .expect("Failed to create backend tag");
    let distributed_tag = db
        .create_or_get_category_tag("distributed")
        .await
        .expect("Failed to create distributed tag");

    let rust_tag_id = rust_tag.id.expect("rust tag missing id");
    let backend_tag_id = backend_tag.id.expect("backend tag missing id");
    let distributed_tag_id = distributed_tag.id.expect("distributed tag missing id");

    db.create_edge(&rust_tag_id, &backend_tag_id, "related_to")
        .await
        .expect("Failed to connect rust->backend");
    db.create_edge(&backend_tag_id, &distributed_tag_id, "related_to")
        .await
        .expect("Failed to connect backend->distributed");

    let results = get_context("rust".to_string(), &db, &vector_db)
        .await
        .expect("Failed to retrieve context");

    assert!(
        results
            .iter()
            .any(|item| item["name"] == "Implement retry policies"),
        "Expected node within two hops to be included"
    );
    assert!(
        !results
            .iter()
            .any(|item| item["description"] == "Use idempotent handlers"),
        "Expected node beyond two hops to be excluded"
    );
}
