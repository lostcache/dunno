use lazydev::vector_db::VectorDB;

#[tokio::test]
async fn test_qdrant_setup() {
    let db = VectorDB::new("mem://")
        .await
        .expect("Failed to init in-memory vector db");
    db.ensure_collection("knowledge", 3)
        .await
        .expect("Failed to create collection");

    db.upsert("knowledge", "a", vec![1.0, 0.0, 0.0])
        .await
        .expect("Failed to upsert vector a");
    db.upsert("knowledge", "b", vec![0.0, 1.0, 0.0])
        .await
        .expect("Failed to upsert vector b");

    let ids = db
        .search("knowledge", &[0.9, 0.1, 0.0], 1)
        .await
        .expect("Failed to search vectors");
    assert_eq!(ids, vec!["a".to_string()]);
}
