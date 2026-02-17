use clap::Parser;
// We assume the crate exposes `args` module with `Args` and `Commands`
use lazydev::args::{Args, Commands};

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
