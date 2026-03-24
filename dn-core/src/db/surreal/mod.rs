//! SurrealDB backend: client, connection, and generic helpers.

mod entities;
mod hierarchy;
mod schema;
mod util;

#[derive(Clone, Debug)]
pub struct DB {
    pub(crate) client: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

impl DB {
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
                Self::new_local(&url, "dunno", "dunno").await.map_err(|e| {
                    if e.to_string().contains("already locked by another process") {
                        anyhow::anyhow!(
                            "Cannot access the database — dn-server appears to be running and has it locked.\n\
                             To use dn and dn-server concurrently, set backend = \"local-server\" in your config\n\
                             and point both tools at a running SurrealDB instance."
                        )
                    } else {
                        e
                    }
                })
            }
            crate::config::StorageBackend::LocalServer => {
                if config.url.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "local-server backend requires `url` (or DUNNO_URL)"
                    ));
                }
                Self::connect_remote(config).await
            }
            crate::config::StorageBackend::Cloud => {
                if config.url.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "cloud backend requires `url` (or DUNNO_URL)"
                    ));
                }
                if config.namespace.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "cloud backend requires `namespace` (or DUNNO_NS)"
                    ));
                }
                if config.database.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "cloud backend requires `database` (or DUNNO_DB)"
                    ));
                }
                if config.username.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "cloud backend requires `username` (or DUNNO_USER)"
                    ));
                }
                if config.password.trim().is_empty() {
                    return Err(anyhow::anyhow!(
                        "cloud backend requires `password` (or DUNNO_PASS)"
                    ));
                }
                Self::connect_remote(config).await
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

    async fn connect_remote(config: &crate::config::Config) -> anyhow::Result<Self> {
        let client = surrealdb::engine::any::connect(&config.url).await?;
        client
            .use_ns(&config.namespace)
            .use_db(&config.database)
            .await?;

        match config.auth_type.as_str() {
            "namespace" => {
                client
                    .signin(surrealdb::opt::auth::Namespace {
                        namespace: config.namespace.clone(),
                        username: config.username.clone(),
                        password: config.password.clone(),
                    })
                    .await?;
            }
            "database" => {
                client
                    .signin(surrealdb::opt::auth::Database {
                        namespace: config.namespace.clone(),
                        database: config.database.clone(),
                        username: config.username.clone(),
                        password: config.password.clone(),
                    })
                    .await?;
            }
            _ => {
                client
                    .signin(surrealdb::opt::auth::Root {
                        username: config.username.clone(),
                        password: config.password.clone(),
                    })
                    .await?;
            }
        }

        let db = Self { client };
        schema::define_schema(&db.client).await?;
        Ok(db)
    }

    /// Deletes all records from all tables.
    pub async fn purge_database(&self) -> anyhow::Result<()> {
        for table in schema::TABLES {
            let sql = format!("DELETE {}", table);
            self.client.query(&sql).await?;
        }
        Ok(())
    }

    // --- Generic Helpers ---

    /// Creates a RELATE edge between two record ids. Public for the generic `dunno link` CLI.
    pub async fn link(&self, from_id: &str, edge_table: &str, to_id: &str) -> anyhow::Result<()> {
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
        let fetched: Option<surrealdb::types::Value> = match self.client.select((table, key)).await
        {
            Ok(value) => value,
            Err(err) if util::is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = util::surreal_to_json(val);
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
            Err(err) if util::is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = util::surreal_to_json(val);
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

        let json = util::surreal_to_json(row);
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
        Ok(util::surreal_to_json(val))
    }
}

#[cfg(test)]
mod tests;

pub use entities::epics::get_epic_context_json;
pub use entities::files::get_file_context_json;
pub use entities::projects::get_project_structure_json;
pub use entities::tasks::get_task_context_json;
