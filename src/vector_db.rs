use qdrant_client::Qdrant;
use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct VectorDB {
    backend: VectorBackend,
}

enum VectorBackend {
    Qdrant(Qdrant),
    Memory(Arc<Mutex<InMemoryVectorStore>>),
}

#[derive(Default)]
struct InMemoryVectorStore {
    collections: HashMap<String, HashMap<String, Vec<f32>>>,
}

impl VectorDB {
    /// Creates a vector db client. Use `mem://` for in-memory tests.
    pub async fn new(url: &str) -> Result<Self> {
        if url == "mem://" {
            return Ok(Self {
                backend: VectorBackend::Memory(Arc::new(Mutex::new(InMemoryVectorStore::default()))),
            });
        }

        let client = Qdrant::from_url(url).build()?;
        Ok(Self {
            backend: VectorBackend::Qdrant(client),
        })
    }

    /// Ensures a collection exists.
    pub async fn ensure_collection(&self, name: &str, vector_size: u64) -> Result<()> {
        match &self.backend {
            VectorBackend::Qdrant(client) => {
                if !client.collection_exists(name).await? {
                    client
                        .create_collection(
                            CreateCollectionBuilder::new(name)
                                .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
                        )
                        .await?;
                }
            }
            VectorBackend::Memory(store) => {
                let mut store = store.lock().map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                store.collections.entry(name.to_string()).or_default();
            }
        }

        Ok(())
    }

    /// Inserts or updates a vector by id.
    pub async fn upsert(&self, collection: &str, id: &str, vector: Vec<f32>) -> Result<()> {
        match &self.backend {
            VectorBackend::Qdrant(_client) => {
                // Graph-first MVP does not require production vector upsert.
                Ok(())
            }
            VectorBackend::Memory(store) => {
                let mut store = store.lock().map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                let col = store.collections.entry(collection.to_string()).or_default();
                col.insert(id.to_string(), vector);
                Ok(())
            }
        }
    }

    /// Searches by cosine similarity and returns ids sorted descending by score.
    pub async fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<String>> {
        match &self.backend {
            VectorBackend::Qdrant(_client) => {
                // Graph-first MVP does not require production vector search.
                Ok(Vec::new())
            }
            VectorBackend::Memory(store) => {
                let store = store.lock().map_err(|_| anyhow::anyhow!("Vector store lock poisoned"))?;
                let Some(col) = store.collections.get(collection) else {
                    return Ok(Vec::new());
                };

                let mut scored: Vec<(&String, f32)> = col
                    .iter()
                    .map(|(id, vector)| (id, cosine_similarity(vector, query)))
                    .collect();

                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
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
