use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::vector_db::VectorDB;

#[tokio::test]
async fn test_ingestion_flow() {
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    let vector_db = VectorDB::new("mem://").await.expect("Failed to init VectorDB");

    let result = add_knowledge(
        "rust".to_string(),
        "mistake".to_string(),
        "Don't unwrap".to_string(),
        &db,
        &vector_db,
    )
    .await;
    assert!(result.is_ok());

    let edges = db.list_edges().await.expect("Failed to list graph edges");
    assert!(
        edges.iter().any(|e| e.relation == "has_tag"),
        "Expected at least one has_tag edge to be created"
    );
}
