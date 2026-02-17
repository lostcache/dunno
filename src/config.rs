use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub surreal_url: String,
    pub qdrant_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            surreal_url: "ws://localhost:8000".to_string(),
            qdrant_url: "http://localhost:6333".to_string(),
        }
    }
}
