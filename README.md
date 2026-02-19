# lazydev

`lazydev` is a Rust CLI that captures coding knowledge and retrieves relevant context for natural-language queries.

It currently focuses on three knowledge types:
- `mistake`
- `style`
- `skill`

Knowledge is stored in SurrealDB as typed records and linked through category-tag graph edges. Context retrieval is currently graph-first (bounded hop traversal + token/category heuristics) with JSON output.

## Current Status

The project is working as an MVP for:
- adding knowledge with `lazydev add`
- retrieving context with `lazydev context`
- returning structured JSON on runtime and parse errors

Automated tests are in place for CLI parsing/help, ingestion, DB CRUD, and context retrieval flow.

## Prerequisites

- Rust toolchain (stable) with `cargo`
- A running SurrealDB endpoint at `ws://localhost:8000`
- (Optional for current behavior) Qdrant at `http://localhost:6333`

## Build

```bash
cargo build
```

Or for an optimized single binary:

```bash
cargo build --release
```

Binary path:
- debug: `target/debug/lazydev`
- release: `target/release/lazydev`

## Quick Start

1. Ensure SurrealDB is reachable at `ws://localhost:8000`.
2. Run:

```bash
cargo run -- add --category rust --type mistake --content "Avoid unwrap in library code"
```

Expected output:

```json
{"status":"ok"}
```

3. Query context:

```bash
cargo run -- context "how to handle rust errors without unwrap"
```

Expected output shape:

```json
{
  "results": [
    {
      "id": "mistake:...",
      "content": "...",
      "category": "...",
      "tags": [],
      "node_type": "mistake"
    }
  ]
}
```

## CLI Usage

Show root help:

```bash
lazydev --help
```

Add knowledge:

```bash
lazydev add --category <CATEGORY> --type <mistake|style|skill> --content "<CONTENT>"
```

Examples:

```bash
lazydev add --category rust --type mistake --content "Avoid unwrap in production code"
lazydev add --category backend --type skill --content "Design resilient APIs"
```

Retrieve context:

```bash
lazydev context "<QUERY>"
```

Example:

```bash
lazydev context "rust error handling best practices"
```

## Output and Error Contract

Success examples:

```json
{"status":"ok"}
```

```json
{"results":[...]}
```

Error examples:

```json
{"status":"error","kind":"cli_parse_error","error":"..."}
```

```json
{"status":"error","kind":"runtime_error","error":"..."}
```

## Configuration

Current defaults are compiled in:
- SurrealDB URL: `ws://localhost:8000`
- Qdrant URL: `http://localhost:6333`

These defaults are defined in `src/config.rs`.

## Development

Run tests:

```bash
cargo test --all
```

Format:

```bash
cargo fmt --all
```

## Known Limitations

- Retrieval is currently graph-first; vector similarity is not yet used for production query ranking.
- Embedding generation is placeholder-based.
- CLI config loading from TOML file is not yet wired into runtime flags.
- Qdrant operations are present but currently non-blocking for the main retrieval flow.

