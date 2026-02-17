use lazydev::config::Config;

#[test]
fn test_config_parsing() {
    let toml_str = r#"
        surreal_url = "ws://localhost:8000"
        qdrant_url = "http://localhost:6333"
    "#;

    let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");

    assert_eq!(config.surreal_url, "ws://localhost:8000");
    assert_eq!(config.qdrant_url, "http://localhost:6333");
}
