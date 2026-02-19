use clap::Parser;
// We assume the crate exposes `args` module with `Args` and `Commands`
use lazydev::args::{Args, Commands};
use serde_json::Value;
use std::process::Command;

#[test]
fn verify_cli_structure() {
    // Check 'add' command structure
    // Expected usage: lazydev add --category <cat> --type <type> --content <content>
    let args = Args::try_parse_from([
        "lazydev",
        "add",
        "--category",
        "rust",
        "--type",
        "mistake",
        "--content",
        "foo",
    ])
    .unwrap();
    match args.command {
        Commands::Add {
            category,
            kind,
            content,
        } => {
            assert_eq!(category, "rust");
            assert_eq!(kind, "mistake"); // 'type' is reserved, so we use 'kind' or similar
            assert_eq!(content, "foo");
        }
        _ => panic!("Expected Add command"),
    }

    // Check 'context' command structure
    // Expected usage: lazydev context "query string"
    let args = Args::try_parse_from(["lazydev", "context", "query string"]).unwrap();
    match args.command {
        Commands::Context { query } => {
            assert_eq!(query, "query string");
        }
        _ => panic!("Expected Context command"),
    }
}

#[test]
fn parse_errors_are_structured_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_lazydev"))
        .output()
        .expect("Failed to execute lazydev binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit for invalid cli input"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be JSON");

    assert_eq!(payload["status"], "error");
    assert_eq!(payload["kind"], "cli_parse_error");
    assert!(
        payload["error"].as_str().is_some_and(|msg| !msg.is_empty()),
        "Expected non-empty parse error message"
    );
}
