#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    #[serde(rename = "local-server")]
    LocalServer,
    Cloud,
}

impl StorageBackend {
    fn parse(value: &str) -> anyhow::Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "local-server" => Ok(Self::LocalServer),
            "cloud" => Ok(Self::Cloud),
            other => Err(anyhow::anyhow!(
                "Invalid backend '{}'. Expected one of: local, local-server, cloud",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub backend: StorageBackend,
    pub local_path: String,
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub auth_type: String,
    pub qdrant_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Local,
            local_path: "~/.local/share/dunno/data.db".to_string(),
            url: String::new(),
            namespace: "dunno".to_string(),
            database: "dunno".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
            auth_type: "root".to_string(),
            qdrant_url: "mem://".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct PartialConfig {
    backend: Option<String>,
    local_path: Option<String>,
    url: Option<String>,
    namespace: Option<String>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    auth_type: Option<String>,
    qdrant_url: Option<String>,
}

impl Config {
    pub fn load(cli_backend: Option<&str>) -> anyhow::Result<Self> {
        let mut config = Self::default();
        // Priority order (lowest to highest):
        // 5. Defaults (already set)

        // 4. Global config
        config.apply_config_file(&Self::global_config_path())?;

        // 3. Local project config
        config.apply_config_file(&Self::local_config_path())?;

        // 2. ENV vars
        config.apply_env_overrides()?;

        // 1. CLI args (highest)
        config.apply_cli_overrides(cli_backend)?;

        Ok(config)
    }

    // Helper for testing - allows specifying custom paths and controlling env overrides
    #[cfg(test)]
    fn load_from_optional_paths(
        cli_backend: Option<&str>,
        global_path: Option<&std::path::Path>,
        local_path: Option<&std::path::Path>,
        skip_env: bool,
    ) -> anyhow::Result<Self> {
        let mut config = Self::default();
        // Priority order (lowest to highest):
        // 5. Defaults (already set)
        if !skip_env {
            // 4. ENV vars
            config.apply_env_overrides()?;
        }
        // 3. Global config
        if let Some(path) = global_path {
            config.apply_config_file(path)?;
        }
        // 2. Local project config
        if let Some(path) = local_path {
            config.apply_config_file(path)?;
        }
        // 1. CLI args (highest)
        config.apply_cli_overrides(cli_backend)?;
        Ok(config)
    }

    pub fn global_config_path() -> std::path::PathBuf {
        let base = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join(".config").join("dunno").join("dunno.toml")
    }

    pub fn local_config_path() -> std::path::PathBuf {
        std::path::PathBuf::from("dunno.toml")
    }

    pub fn local_data_path(&self) -> std::path::PathBuf {
        expand_tilde_path(&self.local_path)
    }

    pub fn redacted_json(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": match self.backend {
                StorageBackend::Local => "local",
                StorageBackend::LocalServer => "local-server",
                StorageBackend::Cloud => "cloud",
            },
            "local_path": self.local_path,
            "url": self.url,
            "namespace": self.namespace,
            "database": self.database,
            "username": self.username,
            "password": if self.password.is_empty() { "" } else { "***redacted***" },
            "auth_type": self.auth_type,
            "qdrant_url": self.qdrant_url,
            "global_config_path": Self::global_config_path(),
            "local_config_path": Self::local_config_path(),
        })
    }

    pub fn formatted(&self) -> String {
        let backend_str = match self.backend {
            StorageBackend::Local => "local",
            StorageBackend::LocalServer => "local-server",
            StorageBackend::Cloud => "cloud",
        };

        let mut output = String::new();
        output.push_str("=== Configuration ===\n\n");
        output.push_str(&format!("Backend: {}\n\n", backend_str));

        match self.backend {
            StorageBackend::Local => {
                output.push_str("--- Local Storage ---\n");
                output.push_str(&format!("Database Path: {}\n", self.local_path));
            }
            StorageBackend::LocalServer | StorageBackend::Cloud => {
                output.push_str(&format!("URL: {}\n", self.url));
                output.push_str(&format!("Namespace: {}\n", self.namespace));
                output.push_str(&format!("Database: {}\n", self.database));
                output.push_str(&format!("Username: {}\n", self.username));
                let password_display = if self.password.is_empty() {
                    "(not set)".to_string()
                } else {
                    "***redacted***".to_string()
                };
                output.push_str(&format!("Password: {}\n", password_display));
                if matches!(self.backend, StorageBackend::Cloud) {
                    output.push_str(&format!("Auth Type: {}\n", self.auth_type));
                }
            }
        }

        output.push_str("\n--- Vector Store ---\n");
        output.push_str(&format!("Qdrant URL: {}\n", self.qdrant_url));

        output.push_str("\n--- Config File Paths ---\n");
        output.push_str(&format!(
            "Global: {}\n",
            Self::global_config_path().display()
        ));
        output.push_str(&format!("Local: {}\n", Self::local_config_path().display()));

        output
    }

    fn apply_config_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let raw = anyhow::Context::context(
            std::fs::read_to_string(path),
            format!("Failed to read config file at {}", path.display()),
        )?;
        let parsed: PartialConfig = anyhow::Context::context(
            toml::from_str(&raw),
            format!("Failed to parse TOML at {}", path.display()),
        )?;
        self.merge_partial(parsed)?;
        Ok(())
    }

    fn apply_env_overrides(&mut self) -> anyhow::Result<()> {
        let pairs = [
            ("DUNNO_BACKEND", std::env::var("DUNNO_BACKEND").ok()),
            ("DUNNO_LOCAL_PATH", std::env::var("DUNNO_LOCAL_PATH").ok()),
            ("DUNNO_URL", std::env::var("DUNNO_URL").ok()),
            ("DUNNO_NS", std::env::var("DUNNO_NS").ok()),
            ("DUNNO_DB", std::env::var("DUNNO_DB").ok()),
            ("DUNNO_USER", std::env::var("DUNNO_USER").ok()),
            ("DUNNO_PASS", std::env::var("DUNNO_PASS").ok()),
            ("DUNNO_AUTH_TYPE", std::env::var("DUNNO_AUTH_TYPE").ok()),
        ];
        self.apply_env_override_pairs(pairs)
    }

    fn apply_env_override_pairs<I>(&mut self, pairs: I) -> anyhow::Result<()>
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
            self.local_path = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_URL") {
            self.url = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_NS") {
            self.namespace = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_DB") {
            self.database = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_USER") {
            self.username = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_PASS") {
            self.password = value.to_string();
        }
        if let Some(value) = map.get("DUNNO_AUTH_TYPE") {
            self.auth_type = value.to_string();
        }
        Ok(())
    }

    fn apply_cli_overrides(&mut self, cli_backend: Option<&str>) -> anyhow::Result<()> {
        if let Some(value) = cli_backend {
            self.backend = StorageBackend::parse(value)?;
        }
        Ok(())
    }

    fn merge_partial(&mut self, partial: PartialConfig) -> anyhow::Result<()> {
        if let Some(backend) = partial.backend {
            self.backend = StorageBackend::parse(&backend)?;
        }
        if let Some(v) = partial.local_path {
            self.local_path = v;
        }
        if let Some(v) = partial.url {
            self.url = v;
        }
        if let Some(v) = partial.namespace {
            self.namespace = v;
        }
        if let Some(v) = partial.database {
            self.database = v;
        }
        if let Some(v) = partial.username {
            self.username = v;
        }
        if let Some(v) = partial.password {
            self.password = v;
        }
        if let Some(v) = partial.auth_type {
            self.auth_type = v;
        }
        if let Some(v) = partial.qdrant_url {
            self.qdrant_url = v;
        }
        Ok(())
    }
}

fn expand_tilde_path(raw: &str) -> std::path::PathBuf {
    if raw == "~" {
        return dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    std::path::Path::new(raw).to_path_buf()
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
            local_path = "~/.local/share/dunno/data.db"
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
        assert_eq!(config.local_path, "~/.local/share/dunno/data.db");
        assert_eq!(config.url, "wss://example.surrealdb.com");
        assert_eq!(config.username, "user");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde_path("~/tmp/dunno");
        let as_str = expanded.to_string_lossy();
        assert!(as_str.contains("tmp/dunno"));
    }

    #[test]
    fn test_load_defaults_when_file_missing() {
        let missing = std::path::PathBuf::from("/tmp/definitely-missing-dunno-config.toml");
        let config = Config::load_from_optional_paths(None, Some(&missing), None, true)
            .expect("load should succeed");
        assert!(matches!(config.backend, StorageBackend::Local));
        assert_eq!(config.qdrant_url, "mem://");
    }

    #[test]
    fn test_env_overrides_apply() {
        let mut config = Config::default();
        config
            .apply_env_override_pairs([
                ("DUNNO_BACKEND", Some("cloud".to_string())),
                ("DUNNO_URL", Some("wss://example.com/rpc".to_string())),
                ("DUNNO_NS", Some("ns1".to_string())),
                ("DUNNO_DB", Some("db1".to_string())),
                ("DUNNO_USER", Some("u1".to_string())),
                ("DUNNO_PASS", Some("p1".to_string())),
                ("DUNNO_LOCAL_PATH", Some("/tmp/override.db".to_string())),
            ])
            .expect("env overrides should apply");

        assert!(matches!(config.backend, StorageBackend::Cloud));
        assert_eq!(config.url, "wss://example.com/rpc");
        assert_eq!(config.local_path, "/tmp/override.db");
    }

    #[test]
    fn test_precedence_cli_over_local_file() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let local_path = std::env::temp_dir().join(format!("dunno-local-config-{ts}.toml"));
        let local_raw = r#"
backend = "cloud"
url = "wss://local.example.surrealdb.com"
namespace = "local"
database = "local"
username = "local"
password = "local"
"#;
        std::fs::write(&local_path, local_raw).expect("should write temp local config");

        // CLI should override local config
        let loaded = Config::load_from_optional_paths(Some("local"), None, Some(&local_path), true)
            .expect("load should succeed");
        assert!(matches!(loaded.backend, StorageBackend::Local));

        let _ = std::fs::remove_file(local_path);
    }

    #[test]
    fn test_local_config_overrides_global() {
        use std::sync::atomic::{AtomicU64, Ordering};

        // Use a counter for uniqueness in parallel tests
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_micros(); // Use microseconds for better uniqueness

        let unique_id = format!("{}-{}", ts, counter);

        let global_path =
            std::env::temp_dir().join(format!("dunno-global-config-{unique_id}.toml"));
        let local_path = std::env::temp_dir().join(format!("dunno-local-config-{unique_id}.toml"));

        let global_raw = r#"
backend = "cloud"
url = "wss://global.example.surrealdb.com"
namespace = "global"
database = "global"
username = "global"
password = "global"
"#;

        let local_raw = r#"
backend = "local"
local_path = "/tmp/local-override.db"
"#;

        std::fs::write(&global_path, global_raw).expect("should write temp global config");
        std::fs::write(&local_path, local_raw).expect("should write temp local config");

        // Local config should override global
        let loaded =
            Config::load_from_optional_paths(None, Some(&global_path), Some(&local_path), true)
                .expect("load should succeed");
        assert!(matches!(loaded.backend, StorageBackend::Local));
        assert_eq!(loaded.local_path, "/tmp/local-override.db");

        let _ = std::fs::remove_file(global_path);
        let _ = std::fs::remove_file(local_path);
    }

    #[test]
    fn test_defaults() {
        let config = Config::default();
        assert_eq!(config.url, "");
        assert_eq!(config.namespace, "dunno");
        assert_eq!(config.database, "dunno");
        assert_eq!(config.username, "root");
        assert_eq!(config.password, "root");
        assert_eq!(config.local_path, "~/.local/share/dunno/data.db");
    }

    #[test]
    fn test_config_partial_override() {
        let toml_str = r#"
            backend = "cloud"
            url = "wss://my-instance.surreal.cloud"
            namespace = "dunno"
            database = "dunno"
        "#;

        let parsed: PartialConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        let mut config = Config::default();
        config.merge_partial(parsed).expect("Failed to merge");

        assert!(matches!(config.backend, StorageBackend::Cloud));
        assert_eq!(config.url, "wss://my-instance.surreal.cloud");
        assert_eq!(config.namespace, "dunno");
        assert_eq!(config.database, "dunno");
        assert_eq!(config.username, "root");
        assert_eq!(config.password, "root");
    }

    #[test]
    fn test_invalid_backend_errors() {
        let mut config = Config::default();
        let err = config
            .apply_env_override_pairs([("DUNNO_BACKEND", Some("invalid".to_string()))])
            .expect_err("invalid backend should fail");
        assert!(err.to_string().contains("Invalid backend"));
    }

    // ============================================================================
    // Happy Path Tests for Config Priority Hierarchy
    // ============================================================================

    #[test]
    fn test_defaults_apply_when_no_config_files_or_env() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let missing_global = std::env::temp_dir().join(format!("dunno-missing-global-{ts}.toml"));
        let missing_local = std::env::temp_dir().join(format!("dunno-missing-local-{ts}.toml"));

        // Skip env to ensure test isolation - we're testing defaults
        let loaded = Config::load_from_optional_paths(
            None,
            Some(&missing_global),
            Some(&missing_local),
            true,
        )
        .expect("load should succeed with defaults");

        // Should have defaults since no files exist and env is skipped
        assert!(matches!(loaded.backend, StorageBackend::Local));
        assert_eq!(loaded.local_path, "~/.local/share/dunno/data.db");
    }

    #[test]
    fn test_global_config_applies_when_no_local() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let global_path = std::env::temp_dir().join(format!("dunno-global-only-{ts}.toml"));
        let missing_local = std::env::temp_dir().join(format!("dunno-missing-local-{ts}.toml"));

        let global_raw = r#"
backend = "cloud"
url = "wss://global-only.example.com"
namespace = "global-ns"
database = "global-db"
username = "global-user"
password = "global-pass"
"#;

        std::fs::write(&global_path, global_raw).expect("should write global config");

        let loaded =
            Config::load_from_optional_paths(None, Some(&global_path), Some(&missing_local), true)
                .expect("load should succeed");

        assert!(matches!(loaded.backend, StorageBackend::Cloud));
        assert_eq!(loaded.url, "wss://global-only.example.com");
        assert_eq!(loaded.namespace, "global-ns");

        let _ = std::fs::remove_file(global_path);
    }

    #[test]
    fn test_local_config_overrides_partial_global_values() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let global_path = std::env::temp_dir().join(format!("dunno-global-partial-{ts}.toml"));
        let local_path = std::env::temp_dir().join(format!("dunno-local-partial-{ts}.toml"));

        let global_raw = r#"
backend = "cloud"
url = "wss://global.example.com"
namespace = "global-ns"
database = "global-db"
username = "global-user"
password = "global-pass"
"#;

        let local_raw = r#"
url = "wss://local.example.com"
namespace = "local-ns"
"#;

        std::fs::write(&global_path, global_raw).expect("should write global config");
        std::fs::write(&local_path, local_raw).expect("should write local config");

        let loaded =
            Config::load_from_optional_paths(None, Some(&global_path), Some(&local_path), true)
                .expect("load should succeed");

        // Local should override specific fields
        assert!(matches!(loaded.backend, StorageBackend::Cloud)); // From global
        assert_eq!(loaded.url, "wss://local.example.com"); // Overridden by local
        assert_eq!(loaded.namespace, "local-ns"); // Overridden by local
        assert_eq!(loaded.database, "global-db"); // From global (not in local)
        assert_eq!(loaded.username, "global-user"); // From global (not in local)

        let _ = std::fs::remove_file(global_path);
        let _ = std::fs::remove_file(local_path);
    }

    #[test]
    fn test_cli_backend_override_preserves_other_config_values() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let local_path = std::env::temp_dir().join(format!("dunno-local-cli-{ts}.toml"));

        let local_raw = r#"
backend = "cloud"
url = "wss://cli-test.example.com"
namespace = "cli-ns"
database = "cli-db"
username = "cli-user"
password = "cli-pass"
local_path = "/tmp/cli-test.db"
"#;

        std::fs::write(&local_path, local_raw).expect("should write local config");

        // CLI overrides backend to local, but cloud values remain from config
        let loaded = Config::load_from_optional_paths(Some("local"), None, Some(&local_path), true)
            .expect("load should succeed");

        assert!(matches!(loaded.backend, StorageBackend::Local)); // CLI override
        assert_eq!(loaded.url, "wss://cli-test.example.com"); // From local config
        assert_eq!(loaded.local_path, "/tmp/cli-test.db"); // From local config

        let _ = std::fs::remove_file(local_path);
    }

    // ============================================================================
    // Edge Case and Failing Tests
    // ============================================================================

    #[test]
    fn test_invalid_toml_format_fails_gracefully() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let invalid_path = std::env::temp_dir().join(format!("dunno-invalid-{ts}.toml"));

        // Invalid TOML - missing closing bracket
        let invalid_raw = r#"
backend = "cloud"
[cloud
url = "wss://example.com"
"#;

        std::fs::write(&invalid_path, invalid_raw).expect("should write invalid config");

        let result = Config::load_from_optional_paths(None, Some(&invalid_path), None, true);
        assert!(result.is_err(), "should fail with invalid TOML");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse TOML") || err_msg.contains("TOML"));

        let _ = std::fs::remove_file(invalid_path);
    }

    #[test]
    fn test_empty_config_file_uses_defaults() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let empty_path = std::env::temp_dir().join(format!("dunno-empty-{ts}.toml"));

        // Empty file
        std::fs::write(&empty_path, "").expect("should write empty config");

        let loaded = Config::load_from_optional_paths(None, Some(&empty_path), None, true)
            .expect("load should succeed with defaults");

        // Should use defaults
        assert!(matches!(loaded.backend, StorageBackend::Local));
        assert_eq!(loaded.local_path, "~/.local/share/dunno/data.db");

        let _ = std::fs::remove_file(empty_path);
    }

    #[test]
    fn test_local_backend_config_with_cloud_in_global() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let global_path = std::env::temp_dir().join(format!("dunno-mixed-{ts}-global.toml"));
        let local_path = std::env::temp_dir().join(format!("dunno-mixed-{ts}-local.toml"));

        let global_raw = r#"
backend = "cloud"
url = "wss://cloud.example.com"
"#;

        let local_raw = r#"
backend = "local"
local_path = "/tmp/mixed.db"
"#;

        std::fs::write(&global_path, global_raw).expect("should write global config");
        std::fs::write(&local_path, local_raw).expect("should write local config");

        let loaded =
            Config::load_from_optional_paths(None, Some(&global_path), Some(&local_path), true)
                .expect("load should succeed");

        // Local backend setting should win
        assert!(matches!(loaded.backend, StorageBackend::Local));
        assert_eq!(loaded.local_path, "/tmp/mixed.db");
        // Connection values from global are still loaded even when backend = local
        assert_eq!(loaded.url, "wss://cloud.example.com");

        let _ = std::fs::remove_file(global_path);
        let _ = std::fs::remove_file(local_path);
    }

    #[test]
    fn test_whitespace_only_config_file_uses_defaults() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let whitespace_path = std::env::temp_dir().join(format!("dunno-whitespace-{ts}.toml"));

        // Only whitespace
        std::fs::write(&whitespace_path, "   \n\n  \t  \n")
            .expect("should write whitespace config");

        let loaded = Config::load_from_optional_paths(None, Some(&whitespace_path), None, true)
            .expect("load should succeed with defaults");

        // Should use defaults
        assert!(matches!(loaded.backend, StorageBackend::Local));

        let _ = std::fs::remove_file(whitespace_path);
    }

    #[test]
    fn test_config_path_methods_return_expected_paths() {
        let global = Config::global_config_path();
        let local = Config::local_config_path();

        // Global should contain .config/dunno/dunno.toml
        let global_str = global.to_string_lossy();
        assert!(global_str.contains(".config"));
        assert!(global_str.contains("dunno"));
        assert!(global_str.ends_with("dunno.toml"));

        // Local should be just dunno.toml in current dir
        let local_str = local.to_string_lossy();
        assert_eq!(local_str, "dunno.toml");
    }

    #[test]
    fn test_qdrant_url_configurable() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let config_path = std::env::temp_dir().join(format!("dunno-qdrant-{ts}.toml"));

        let config_raw = r#"
qdrant_url = "http://localhost:6333"
"#;

        std::fs::write(&config_path, config_raw).expect("should write config");

        let loaded = Config::load_from_optional_paths(None, Some(&config_path), None, true)
            .expect("load should succeed");

        assert_eq!(loaded.qdrant_url, "http://localhost:6333");

        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn test_invalid_qdrant_url_in_config_is_accepted() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let config_path = std::env::temp_dir().join(format!("dunno-qdrant-invalid-{ts}.toml"));

        // Any string is accepted for qdrant_url (validation happens elsewhere)
        let config_raw = r#"
qdrant_url = "not-a-valid-url"
"#;

        std::fs::write(&config_path, config_raw).expect("should write config");

        let loaded = Config::load_from_optional_paths(None, Some(&config_path), None, true)
            .expect("load should succeed");

        assert_eq!(loaded.qdrant_url, "not-a-valid-url");

        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn test_partial_cloud_config_preserves_defaults() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_millis();
        let config_path = std::env::temp_dir().join(format!("dunno-partial-{ts}.toml"));

        let config_raw = r#"
backend = "cloud"
url = "wss://partial.example.com"
"#;

        std::fs::write(&config_path, config_raw).expect("should write config");

        let loaded = Config::load_from_optional_paths(None, Some(&config_path), None, true)
            .expect("load should succeed");

        assert!(matches!(loaded.backend, StorageBackend::Cloud));
        assert_eq!(loaded.url, "wss://partial.example.com");
        // Other fields should use defaults
        assert_eq!(loaded.username, "root");
        assert_eq!(loaded.password, "root");
        assert_eq!(loaded.namespace, "dunno");
        assert_eq!(loaded.database, "dunno");

        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn test_formatted_output_local_backend() {
        let config = Config::default();
        let formatted = config.formatted();

        assert!(formatted.contains("Backend: local"));
        assert!(formatted.contains("Local Storage"));
        assert!(formatted.contains("Database Path:"));
        assert!(formatted.contains("Config File Paths"));
    }

    #[test]
    fn test_formatted_output_cloud_backend() {
        let mut config = Config::default();
        config.backend = StorageBackend::Cloud;
        config.url = "wss://test.surrealdb.com".to_string();
        config.namespace = "test_ns".to_string();
        config.database = "test_db".to_string();
        config.username = "test_user".to_string();
        config.password = "secret_password".to_string();

        let formatted = config.formatted();

        assert!(formatted.contains("Backend: cloud"));
        assert!(formatted.contains("wss://test.surrealdb.com"));
        assert!(formatted.contains("test_ns"));
        assert!(formatted.contains("test_db"));
        assert!(formatted.contains("test_user"));
        assert!(formatted.contains("***redacted***"));
    }
}
