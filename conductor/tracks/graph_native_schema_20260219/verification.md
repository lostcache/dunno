# Verification Plan: Graph-Native Schema Redesign

## Automated Tests

- [ ] `cargo test --all` — all unit and integration tests pass
- [ ] `cargo fmt --all --check` — formatting clean
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — no warnings

## Manual Verification

### 1. Structural Hierarchy CRUD (RELATE-based)

```bash
# Create hierarchy
dunno project create "Test App" "A test application"
# Returns project:<id>

dunno module create --project-id project:<id> "Auth" "Authentication module"
# Returns module:<id>

dunno submodule create --module-id module:<id> "OAuth" "OAuth2 providers"
# Returns submodule:<id>

dunno file create --parent-id submodule:<id> "oauth.rs" "src/auth/oauth.rs"
# Returns file:<id>

dunno task create --module-id module:<id> "Implement JWT" "Add token support"
# Returns task:<id>

dunno subtask create --task-id task:<id> "Write tests" "Unit tests for JWT"
# Returns subtask:<id>
```

**Verify:** Each create returns JSON with an `id` field. No FK fields in the output (no `project_id`, `module_id`, etc.).

### 2. List Children via Graph Traversal

```bash
dunno module list
# Should list all modules

dunno submodule list --module-id module:<id>
# Should list only submodules under that module

dunno file list --submodule-id submodule:<id>
# Should list only files under that submodule

dunno subtask list --task-id task:<id>
# Should list only subtasks under that task
```

**Verify:** List commands return correct children, not all records.

### 3. Knowledge Linking (RELATE-based)

```bash
# Link a style rule to the project (global)
dunno add --category rust --type style --content "Use explicit error types" --link-to project:<id>

# Link a mistake to the task
dunno add --category security --type mistake --content "Do not log raw passwords" --link-to task:<id>

# Link a security detail to the module
dunno add --category auth --type security --content "Validate JWT expiry" --link-to module:<id>
```

**Verify:** Each `add` returns `{"status":"ok"}`.

### 4. Context Retrieval (single-query)

```bash
# Task context: should include task-linked mistake + module-linked security + project-linked style
dunno context --task-id task:<id>

# File context: should include project-linked style (inherited through submodule -> module -> project)
dunno context --file-id file:<id>

# Subtask context: should include all 4 levels (subtask -> task -> module -> project)
dunno context --subtask-id subtask:<id>
```

**Verify:** Each context response contains knowledge nodes from the queried node AND all ancestors. No duplicates.

### 5. Verify KnowledgeEdge and CategoryTag are Gone

```bash
# These should NOT appear in any output
# Internally: no knowledge_edge or category_tag tables should exist
dunno config show
# Should work as before (config is unchanged)
```

### 6. No FK Fields in JSON Output

```bash
dunno module list
# Module JSON should NOT contain "project_id"

dunno task list
# Task JSON should NOT contain "module_id"

dunno file list
# File JSON should NOT contain "module_id" or "submodule_id"
```

**Verify:** All JSON output contains only node-local data. Parent relationships are in the graph, not in the record.
