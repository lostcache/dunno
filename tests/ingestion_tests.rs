use lazydev::db::DB;
use lazydev::vector_db::VectorDB;
use lazydev::ingest::add_knowledge;

#[tokio::test]
async fn test_ingestion_flow() {
    // Setup DBs (using in-memory Surreal)
    let db = DB::new("mem://").await.expect("Failed to init SurrealDB");
    
    // For VectorDB, we need a URL. If no Qdrant, we might panic or we need to mock.
    // Let's assume we can pass a dummy URL and `add_knowledge` won't fail 
    // unless it tries to connect/upsert which we haven't implemented yet fully.
    // Or we use a mock. 
    // Since VectorDB::new connects immediately, this test will fail if Qdrant isn't running.
    // We should skip if connection fails?
    
    let vector_db_res = VectorDB::new("http://localhost:6333").await;
    if let Ok(vector_db) = vector_db_res {
        let result = add_knowledge(
            "rust".to_string(),
            "mistake".to_string(),
            "Don't unwrap".to_string(),
            &db,
            &vector_db
        ).await;
        
        assert!(result.is_ok());
    } else {
        println!("Skipping ingestion test: Qdrant not available");
    }
}
