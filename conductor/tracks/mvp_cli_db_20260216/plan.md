# Track Plan: MVP - CLI & Database Integration

## Phase 1: Project Initialization & CLI Skeleton [checkpoint: 377d4cf]
- [x] Task: Initialize Rust project and configure dependencies (clap, serde, tokio, etc.). [a0a7e30]
- [x] Task: Implement the `clap` CLI structure with `add` and `context` subcommands. [fbccbba]
- [x] Task: Create a `config` module to handle TOML configuration (e.g., DB URLs). [199706f]
- [x] Task: Conductor - User Manual Verification 'Phase 1: Project Initialization & CLI Skeleton' (Protocol in workflow.md) [377d4cf]

## Phase 2: Data Modeling & Database Setup [checkpoint: abce752]
- [x] Task: Define Rust structs for `Mistake`, `StyleRule`, and `Skill` (deriving Serialize/Deserialize). [ed47a16]
- [x] Task: specific set up SurrealDB connection and implement basic CRUD operations for the models. [ff6c8e5]
- [x] Task: specific set up Qdrant connection and create a collection for knowledge embeddings. [cefc8b7]
- [x] Task: Conductor - User Manual Verification 'Phase 2: Data Modeling & Database Setup' (Protocol in workflow.md) [abce752]

## Phase 3: Core Logic - Ingestion (Adding Data) [checkpoint: d31d15a]
- [x] Task: Implement the `lazydev add` logic to: [bc7b423]
    1.  Generate an embedding for the content (using a placeholder or simple local model).
    2.  Store the structured data in SurrealDB.
    3.  Store the embedding + ID in Qdrant.
- [x] Task: Write unit tests for the ingestion logic (mocking DBs if possible). [d31d15a]
- [x] Task: Conductor - User Manual Verification 'Phase 3: Core Logic - Ingestion' (Protocol in workflow.md) [d31d15a]

## Phase 4: Core Logic - Retrieval (Context)
- [x] Task: Implement the `lazydev context` logic to: [bc0812f]
    1.  Parse user query into graph seeds (category/tag/token heuristics).
    2.  Traverse SurrealDB graph edges (bounded hop search).
    3.  Fetch connected knowledge nodes (`mistake`, `style_rule`, `skill`).
    4.  Format the output as JSON.
- [x] Task: Write integration tests for the full flow (Add -> Context). [bc0812f]
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Core Logic - Retrieval' (Protocol in workflow.md)

## Phase 5: Polish & Error Handling
- [x] Task: Ensure all error paths return structured JSON (as per guidelines). [a30c131]
- [x] Task: Add helpful help messages to the CLI.
- [ ] Task: Verify the single-binary build process.
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Polish & Error Handling' (Protocol in workflow.md)

## Phase 6: Agent-Centric Knowledge Graph (Project/Module/Task/Todo)
- [ ] Task: Define and implement graph entities for `Project`, `Module`, `Task`, `TodoItem`, and append-only `TaskUpdate`.
- [ ] Task: Implement hierarchy edges (`project -> module -> task`) and todo edges (`project -> has_todo -> todo_item`, optional `todo_item -> maps_to -> task`).
- [ ] Task: Implement project-global guidance links for style and mistakes (`global_should_follow`, `global_must_avoid`).
- [ ] Task: Implement dynamic agent mistake logging (code/logical) with project/module/task scoping and append-only persistence.
- [ ] Task: Implement task-context retrieval by `task_id` including inherited project-global guidance and runtime learnings.
- [ ] Task: Add CLI commands for hierarchy management, todo operations, global guidance linking, and mistake logging/listing.
- [ ] Task: Write integration tests for full agent workflow (todo select -> task context -> append updates/mistakes -> subsequent retrieval).
- [ ] Task: Conductor - User Manual Verification 'Phase 6: Agent-Centric Knowledge Graph' (Protocol in workflow.md)
