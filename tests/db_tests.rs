use lazydev::db::DB;
use lazydev::models::Mistake;

#[tokio::test]
async fn test_surreal_crud() {
    // Attempt to initialize DB (in-memory for testing)
    let db = DB::new("mem://").await.expect("Failed to init DB");

    // Test data
    let mistake = Mistake {
        id: None,
        content: "Using unwrap in production code".to_string(),
        category: "rust".to_string(),
        tags: vec!["safety".to_string()],
    };

    // CRUD: Create
    let created: Mistake = db
        .create_mistake(&mistake)
        .await
        .expect("Failed to create mistake");
    assert!(created.id.is_some());
    assert_eq!(created.content, mistake.content);

    // CRUD: Read
    let id = created.id.as_ref().unwrap();
    let fetched: Option<Mistake> = db.get_mistake(id).await.expect("Failed to fetch mistake");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().content, mistake.content);

    let tag = db
        .create_or_get_category_tag("rust")
        .await
        .expect("Failed to create tag");
    assert_eq!(tag.normalized, "rust");

    let tag_id = tag.id.expect("Tag should have id");
    db.create_edge(id, &tag_id, "has_tag")
        .await
        .expect("Failed to create edge");
    let edges = db.list_edges().await.expect("Failed to list edges");
    assert!(edges.iter().any(|e| e.relation == "has_tag"));
}
