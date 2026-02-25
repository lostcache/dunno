//! SurrealDB backend: client, connection, and generic helpers.

mod convert;
mod entities;
mod schema;

use convert::{is_missing_table_error, surreal_to_json};

#[derive(Clone)]
pub struct DB {
    pub(crate) client: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

/// Full structural hierarchy for a node (used when creating reverse knowledge edges).
#[allow(dead_code)]
pub(crate) struct StructuralHierarchy {
    pub(crate) project_id: Option<String>,
    pub(crate) module_id: Option<String>,
    pub(crate) submodule_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) subtask_id: Option<String>,
}

impl DB {
    /// TODO: try and unify new methods.
    /// Creates a new SurrealDB client and selects the default namespace/database.
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(url).await?;

        if !url.starts_with("mem:") {
            client
                .signin(surrealdb::opt::auth::Root {
                    username: "root".to_string(),
                    password: "root".to_string(),
                })
                .await?;
        }
        client.use_ns("dunno").use_db("dunno").await?;
        let db = Self { client };
        schema::define_schema(&db.client).await?;
        Ok(db)
    }

    /// Creates a DB client from runtime config (local embedded or cloud).
    pub async fn from_config(config: &crate::config::Config) -> anyhow::Result<Self> {
        match config.backend {
            crate::config::StorageBackend::Local => {
                let path = config.local_data_path();
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let url = format!("surrealkv://{}", path.to_string_lossy());
                Self::new_local(&url, "dunno", "dunno").await
            }
            crate::config::StorageBackend::Cloud => {
                let cloud = &config.cloud;
                if cloud.url.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.url` (or DUNNO_CLOUD_URL)"
                    ));
                }
                if cloud.namespace.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.namespace` (or DUNNO_CLOUD_NS)"
                    ));
                }
                if cloud.database.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.database` (or DUNNO_CLOUD_DB)"
                    ));
                }
                if cloud.username.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.username` (or DUNNO_CLOUD_USER)"
                    ));
                }
                if cloud.password.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "Cloud backend requires `cloud.password` (or DUNNO_CLOUD_PASS)"
                    ));
                }
                Self::connect_cloud(cloud).await
            }
        }
    }

    async fn new_local(url: &str, namespace: &str, database: &str) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(url).await?;
        if url.starts_with("ws://")
            || url.starts_with("wss://")
            || url.starts_with("http://")
            || url.starts_with("https://")
        {
            client
                .signin(surrealdb::opt::auth::Root {
                    username: "root".to_string(),
                    password: "root".to_string(),
                })
                .await?;
        }
        client.use_ns(namespace).use_db(database).await?;
        let db = Self { client };
        schema::define_schema(&db.client).await?;
        Ok(db)
    }

    async fn connect_cloud(cloud: &crate::config::CloudConfig) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(&cloud.url).await?;
        client
            .use_ns(&cloud.namespace)
            .use_db(&cloud.database)
            .await?;

        match cloud.auth_type.as_str() {
            "namespace" => {
                client
                    .signin(surrealdb::opt::auth::Namespace {
                        namespace: cloud.namespace.clone(),
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
            "database" => {
                client
                    .signin(surrealdb::opt::auth::Database {
                        namespace: cloud.namespace.clone(),
                        database: cloud.database.clone(),
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
            _ => {
                client
                    .signin(surrealdb::opt::auth::Root {
                        username: cloud.username.clone(),
                        password: cloud.password.clone(),
                    })
                    .await?;
            }
        }

        let db = Self { client };
        schema::define_schema(&db.client).await?;
        Ok(db)
    }

    // --- Generic Helpers ---

    /// Creates a RELATE edge between two record ids.
    pub(crate) async fn relate(
        &self,
        from_id: &str,
        edge_table: &str,
        to_id: &str,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "LET $f = type::record($from); \
             LET $t = type::record($to); \
             RELATE $f->{edge_table}->$t;"
        );
        self.client
            .query(&sql)
            .bind(("from", from_id.to_string()))
            .bind(("to", to_id.to_string()))
            .await?;
        Ok(())
    }

    pub(crate) async fn get_record<T: serde::de::DeserializeOwned>(
        &self,
        table: &str,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<surrealdb::types::Value> =
            match self.client.select((table, key)).await {
                Ok(value) => value,
                Err(err) if is_missing_table_error(&err) => None,
                Err(err) => return Err(err.into()),
            };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn list_records<T: serde::de::DeserializeOwned>(
        &self,
        table: &str,
    ) -> anyhow::Result<Vec<T>> {
        let fetched: Vec<surrealdb::types::Value> = match self.client.select(table).await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Runs a graph traversal query and extracts a nested array.
    pub(crate) async fn query_graph_list<T: serde::de::DeserializeOwned>(
        &self,
        sql: &str,
        bind_key: &str,
        bind_val: String,
        field: &str,
    ) -> anyhow::Result<Vec<T>> {
        let mut response = self
            .client
            .query(sql)
            .bind((bind_key.to_string(), bind_val))
            .await?;
        let row: Option<surrealdb::types::Value> = response.take(0)?;
        let row = match row {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let json = surreal_to_json(row);
        let items = match json.get(field) {
            Some(serde_json::Value::Array(outer)) => {
                let mut flat = Vec::new();
                for elem in outer {
                    match elem {
                        serde_json::Value::Array(inner) => {
                            for item in inner {
                                flat.push(serde_json::from_value(item.clone())?);
                            }
                        }
                        _ => {
                            flat.push(serde_json::from_value(elem.clone())?);
                        }
                    }
                }
                flat
            }
            _ => Vec::new(),
        };
        Ok(items)
    }

    /// Runs a raw SurrealQL query with a single string binding.
    /// Returns the result at the given statement index as a serde_json::Value.
    pub async fn query_raw_json(
        &self,
        sql: &str,
        key: &str,
        value: String,
        take_index: usize,
    ) -> anyhow::Result<serde_json::Value> {
        let mut response = self
            .client
            .query(sql)
            .bind((key.to_string(), value))
            .await?;
        let val: surrealdb::types::Value = response.take(take_index)?;
        Ok(surreal_to_json(val))
    }
}

#[cfg(test)]
mod tests;
