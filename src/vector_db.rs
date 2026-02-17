use qdrant_client::Qdrant;
use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};
use anyhow::Result;

pub struct VectorDB {
    client: Qdrant,
}

impl VectorDB {
    pub async fn new(url: &str) -> Result<Self> {
        let client = Qdrant::from_url(url).build()?;
        Ok(Self { client })
    }

    pub async fn ensure_collection(&self, name: &str, vector_size: u64) -> Result<()> {
        if !self.client.collection_exists(name).await? {
            self.client.create_collection(
                CreateCollectionBuilder::new(name)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine))
            ).await?;
        }
        Ok(())
    }
}
