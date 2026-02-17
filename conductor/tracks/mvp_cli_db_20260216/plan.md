# Track Plan: MVP - CLI & Database Integration

## Phase 1: Project Initialization & CLI Skeleton
- [x] Task: Initialize Rust project and configure dependencies (clap, serde, tokio, etc.). [a0a7e30]
- [x] Task: Implement the `clap` CLI structure with `add` and `context` subcommands. [fbccbba]
- [x] Task: Create a `config` module to handle TOML configuration (e.g., DB URLs). [199706f]
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Project Initialization & CLI Skeleton' (Protocol in workflow.md)

## Phase 2: Data Modeling & Database Setup
- [ ] Task: Define Rust structs for `Mistake`, `StyleRule`, and `Skill` (deriving Serialize/Deserialize).
- [ ] Task: specific set up SurrealDB connection and implement basic CRUD operations for the models.
- [ ] Task: specific set up Qdrant connection and create a collection for knowledge embeddings.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Data Modeling & Database Setup' (Protocol in workflow.md)

## Phase 3: Core Logic - Ingestion (Adding Data)
- [ ] Task: Implement the `lazydev add` logic to:
    1.  Generate an embedding for the content (using a placeholder or simple local model).
    2.  Store the structured data in SurrealDB.
    3.  Store the embedding + ID in Qdrant.
- [ ] Task: Write unit tests for the ingestion logic (mocking DBs if possible).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Core Logic - Ingestion' (Protocol in workflow.md)

## Phase 4: Core Logic - Retrieval (Context)
- [ ] Task: Implement the `lazydev context` logic to:
    1.  Generate an embedding for the user's query.
    2.  Query Qdrant for nearest neighbors.
    3.  Fetch full details from SurrealDB using the IDs from Qdrant.
    4.  Format the output as JSON.
- [ ] Task: Write integration tests for the full flow (Add -> Context).
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Core Logic - Retrieval' (Protocol in workflow.md)

## Phase 5: Polish & Error Handling
- [ ] Task: Ensure all error paths return structured JSON (as per guidelines).
- [ ] Task: Add helpful help messages to the CLI.
- [ ] Task: Verify the single-binary build process.
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Polish & Error Handling' (Protocol in workflow.md)
