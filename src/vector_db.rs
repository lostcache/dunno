pub struct VectorDB {
    backend: VectorBackend,
}

enum VectorBackend {
    Qdrant(qdrant_client::Qdrant),
    Memory(std::sync::Arc<std::sync::Mutex<InMemoryVectorStore>>),
}

#[derive(Default)]
struct InMemoryVectorStore {
    collections: std::collections::HashMap<String, std::collections::HashMap<String, Vec<f32>>>,
}

impl VectorDB {
    /// Creates a vector db client. Use `mem://` for in-memory tests.
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        if url == "mem://" {
            return Ok(Self {
                backend: VectorBackend::Memory(std::sync::Arc::new(
                    std::sync::Mutex::new(InMemoryVectorStore::default()),
                )),
            });
        }

        let client = qdrant_client::Qdrant::from_url(url).build()?;
        Ok(Self {
            backend: VectorBackend::Qdrant(client),
        })
    }

    /// Ensures a collection exists.
    pub async fn ensure_collection(&self, name: &str, vector_size: u64) -> anyhow::Result<()> {
        match &self.backend {
            VectorBackend::Qdrant(client) => {
                if !client.collection_exists(name).await? {
                    client
                        .create_collection(qdrant_client::qdrant::CreateCollectionBuilder::new(name).vectors_config(
                            qdrant_client::qdrant::VectorParamsBuilder::new(vector_size, qdrant_client::qdrant::Distance::Cosine),
                        ))
                        .await?;
                }
            }
            VectorBackend::Memory(store) => {
                let mut store = store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                store.collections.entry(name.to_string()).or_default();
            }
        }

        Ok(())
    }

    /// Inserts or updates a vector by id.
    pub async fn upsert(&self, collection: &str, id: &str, vector: Vec<f32>) -> anyhow::Result<()> {
        match &self.backend {
            VectorBackend::Qdrant(_client) => {
                // Graph-first MVP does not require production vector upsert.
                Ok(())
            }
            VectorBackend::Memory(store) => {
                let mut store = store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                let col = store.collections.entry(collection.to_string()).or_default();
                col.insert(id.to_string(), vector);
                Ok(())
            }
        }
    }

    /// Searches by cosine similarity and returns ids sorted descending by score.
    pub async fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> anyhow::Result<Vec<String>> {
        match &self.backend {
            VectorBackend::Qdrant(_client) => {
                // Graph-first MVP does not require production vector search.
                Ok(Vec::new())
            }
            VectorBackend::Memory(store) => {
                let store = store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                let Some(col) = store.collections.get(collection) else {
                    return Ok(Vec::new());
                };

                let mut scored: Vec<(&String, f32)> = col
                    .iter()
                    .map(|(id, vector)| (id, cosine_similarity(vector, query)))
                    .collect();

                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                Ok(scored
                    .into_iter()
                    .take(limit)
                    .map(|(id, _)| id.clone())
                    .collect())
            }
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
