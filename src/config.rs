use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    Cloud,
}

impl StorageBackend {
    fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            other => Err(anyhow::anyhow!(
                "Invalid backend '{}'. Expected one of: local, cloud",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub path: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            path: "~/.local/share/dunno/data.db".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub auth_type: String,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            namespace: String::new(),
            database: String::new(),
            username: "root".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub backend: StorageBackend,
    pub local: LocalConfig,
    pub cloud: CloudConfig,
    pub qdrant_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Local,
            local: LocalConfig::default(),
            cloud: CloudConfig::default(),
            qdrant_url: "mem://".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialConfig {
    backend: Option<String>,
    local: Option<PartialLocalConfig>,
    cloud: Option<PartialCloudConfig>,
    qdrant_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialLocalConfig {
    path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialCloudConfig {
    url: Option<String>,
    namespace: Option<String>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    auth_type: Option<String>,
}

impl Config {
    pub fn load(cli_backend: Option<&str>) -> Result<Self> {
        Self::load_from_path(cli_backend, &Self::config_file_path())
    }

    fn load_from_path(cli_backend: Option<&str>, config_path: &Path) -> Result<Self> {
        let mut config = Self::default();
        config.apply_config_file(config_path)?;
        config.apply_env_overrides()?;
        config.apply_cli_overrides(cli_backend)?;
        Ok(config)
    }

    pub fn config_file_path() -> PathBuf {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".config").join("dunno").join("config.toml")
    }

    pub fn local_data_path(&self) -> PathBuf {
        expand_tilde_path(&self.local.path)
    }

    pub fn redacted_json(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": match self.backend {
                StorageBackend::Local => "local",
                StorageBackend::Cloud => "cloud",
            },
            "local": {
                "path": self.local.path,
            },
            "cloud": {
                "url": self.cloud.url,
                "namespace": self.cloud.namespace,
                "database": self.cloud.database,
                "username": self.cloud.username,
                "password": if self.cloud.password.is_empty() { "" } else { "***redacted***" },
                "auth_type": self.cloud.auth_type,
            },
            "qdrant_url": self.qdrant_url,
            "config_path": Self::config_file_path(),
        })
    }

    fn apply_config_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {}", path.display()))?;
        let parsed: PartialConfig = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse TOML at {}", path.display()))?;
        self.merge_partial(parsed)?;
        Ok(())
    }

    fn apply_env_overrides(&mut self) -> Result<()> {
        let pairs = [
            ("DUNNO_BACKEND", env::var("DUNNO_BACKEND").ok()),
            ("DUNNO_LOCAL_PATH", env::var("DUNNO_LOCAL_PATH").ok()),
            ("DUNNO_CLOUD_URL", env::var("DUNNO_CLOUD_URL").ok()),
            ("DUNNO_CLOUD_NS", env::var("DUNNO_CLOUD_NS").ok()),
            ("DUNNO_CLOUD_DB", env::var("DUNNO_CLOUD_DB").ok()),
            ("DUNNO_CLOUD_USER", env::var("DUNNO_CLOUD_USER").ok()),
            ("DUNNO_CLOUD_PASS", env::var("DUNNO_CLOUD_PASS").ok()),
            (
                "DUNNO_CLOUD_AUTH_TYPE",
                env::var("DUNNO_CLOUD_AUTH_TYPE").ok(),
            ),
        ];
        self.apply_env_override_pairs(pairs)
    }

    fn apply_env_override_pairs<I>(&mut self, pairs: I) -> Result<()>
    where
        I: IntoIterator<Item = (&'static str, Option<String>)>,
    {
        let mut map = std::collections::HashMap::new();
        for (key, value) in pairs {
            if let Some(value) = value {
                map.insert(key, value);
            }
        }

        if let Some(value) = map.get("DUNNO_BACKEND") {
            self.backend = StorageBackend::parse(value)?;
        }
        if let Some(value) = map.get("DUNNO_LOCAL_PATH") {
            self.local.path = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_URL") {
            self.cloud.url = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_NS") {
            self.cloud.namespace = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_DB") {
            self.cloud.database = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_USER") {
            self.cloud.username = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_PASS") {
            self.cloud.password = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_CLOUD_AUTH_TYPE") {
            self.cloud.auth_type = value.to_string();
        }
        Ok(())
    }

    fn apply_cli_overrides(&mut self, cli_backend: Option<&str>) -> Result<()> {
        if let Some(value) = cli_backend {
            self.backend = StorageBackend::parse(value)?;
        }
        Ok(())
    }

    fn merge_partial(&mut self, partial: PartialConfig) -> Result<()> {
        if let Some(backend) = partial.backend {
            self.backend = StorageBackend::parse(&backend)?;
        }
        if let Some(local) = partial.local
            && let Some(path) = local.path
        {
            self.local.path = path;
        }
        if let Some(cloud) = partial.cloud {
            if let Some(url) = cloud.url {
                self.cloud.url = url;
            }
            if let Some(namespace) = cloud.namespace {
                self.cloud.namespace = namespace;
            }
            if let Some(database) = cloud.database {
                self.cloud.database = database;
            }
            if let Some(username) = cloud.username {
                self.cloud.username = username;
            }
            if let Some(password) = cloud.password {
                self.cloud.password = password;
            }
            if let Some(auth_type) = cloud.auth_type {
                self.cloud.auth_type = auth_type;
            }
        }
        if let Some(qdrant_url) = partial.qdrant_url {
            self.qdrant_url = qdrant_url;
        }
        Ok(())
    }
}

fn expand_tilde_path(raw: &str) -> PathBuf {
    if raw == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    Path::new(raw).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_config_parsing_new_format() {
        let toml_str = r#"
            backend = "cloud"
            qdrant_url = "http://localhost:6333"
            [local]
            path = "~/.local/share/dunno/data.db"
            [cloud]
            url = "wss://example.surrealdb.com"
            namespace = "dunno"
            database = "dunno"
            username = "user"
            password = "pass"
        "#;

        let parsed: PartialConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        let mut config = Config::default();
        config
            .merge_partial(parsed)
            .expect("Failed to merge partial config");

        assert!(matches!(config.backend, StorageBackend::Cloud));
        assert_eq!(config.local.path, "~/.local/share/dunno/data.db");
        assert_eq!(config.cloud.url, "wss://example.surrealdb.com");
        assert_eq!(config.cloud.username, "user");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde_path("~/tmp/dunno");
        let as_str = expanded.to_string_lossy();
        assert!(as_str.contains("tmp/dunno"));
    }

    #[test]
    fn test_load_defaults_when_file_missing() {
        let missing = PathBuf::from("/tmp/definitely-missing-dunno-config.toml");
        let config = Config::load_from_path(None, &missing).expect("load should succeed");
        assert!(matches!(config.backend, StorageBackend::Local));
        assert_eq!(config.qdrant_url, "mem://");
    }

    #[test]
    fn test_env_overrides_apply() {
        let mut config = Config::default();
        config
            .apply_env_override_pairs([
                ("DUNNO_BACKEND", Some("cloud".to_string())),
                ("DUNNO_CLOUD_URL", Some("wss://example.com/rpc".to_string())),
                ("DUNNO_CLOUD_NS", Some("ns1".to_string())),
                ("DUNNO_CLOUD_DB", Some("db1".to_string())),
                ("DUNNO_CLOUD_USER", Some("u1".to_string())),
                ("DUNNO_CLOUD_PASS", Some("p1".to_string())),
                ("DUNNO_LOCAL_PATH", Some("/tmp/override.db".to_string())),
            ])
            .expect("env overrides should apply");

        assert!(matches!(config.backend, StorageBackend::Cloud));
        assert_eq!(config.cloud.url, "wss://example.com/rpc");
        assert_eq!(config.local.path, "/tmp/override.db");
    }

    #[test]
    fn test_precedence_cli_over_file() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let tmp_path = std::env::temp_dir().join(format!("dunno-config-{ts}.toml"));
        let raw = r#"
backend = "cloud"
[cloud]
url = "wss://example.surrealdb.com"
namespace = "dunno"
database = "dunno"
username = "user"
password = "pass"
"#;
        fs::write(&tmp_path, raw).expect("should write temp config");

        let loaded = Config::load_from_path(Some("local"), &tmp_path).expect("load should succeed");
        assert!(matches!(loaded.backend, StorageBackend::Local));

        let _ = fs::remove_file(tmp_path);
    }

    #[test]
    fn test_cloud_defaults_match_surrealdb() {
        let config = Config::default();
        assert_eq!(config.cloud.url, "");
        assert_eq!(config.cloud.namespace, "");
        assert_eq!(config.cloud.database, "");
        assert_eq!(config.cloud.username, "root");
        assert_eq!(config.cloud.password, "root");
    }

    #[test]
    fn test_cloud_config_partial_override() {
        let toml_str = r#"
            backend = "cloud"
            [cloud]
            url = "wss://my-instance.surreal.cloud"
            namespace = "dunno"
            database = "dunno"
        "#;

        let parsed: PartialConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        let mut config = Config::default();
        config.merge_partial(parsed).expect("Failed to merge");

        assert!(matches!(config.backend, StorageBackend::Cloud));
        assert_eq!(config.cloud.url, "wss://my-instance.surreal.cloud");
        assert_eq!(config.cloud.namespace, "dunno");
        assert_eq!(config.cloud.database, "dunno");
        assert_eq!(config.cloud.username, "root");
        assert_eq!(config.cloud.password, "root");
    }

    #[test]
    fn test_invalid_backend_errors() {
        let mut config = Config::default();
        let err = config
            .apply_env_override_pairs([("DUNNO_BACKEND", Some("invalid".to_string()))])
            .expect_err("invalid backend should fail");
        assert!(err.to_string().contains("Invalid backend"));
    }
}
