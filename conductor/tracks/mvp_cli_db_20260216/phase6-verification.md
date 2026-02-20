# Phase 6: Agent-Centric Knowledge Graph — Manual Verification Plan

## Automated Tests

All 15 tests passed (14 unit + 1 integration). Command: `CI=true cargo test`

### Shell Tests (Local Persistence)

All 12 manual steps below are now automated as shell scripts in `tests/sh/`. The suite tests all three configuration methods (env vars, config file, CLI flags), config precedence, and cross-method data persistence.

```bash
./tests/sh/run_all.sh        # 5 suites, 127 assertions
```

## Prerequisites

- Build the binary: `cargo build`
- No external SurrealDB server is required — the local backend uses embedded `surrealkv://`.

## Steps

### 1. Create a Project

```bash
cargo run -- project create "MyApp" "An example application"
```

**Confirm:** JSON response with `id`, `name`, and `description` fields. Note the `id` value (e.g., `project:xxx`).

### 2. Create a Module under the Project

```bash
cargo run -- module create "<project_id>" "Auth" "Authentication module"
```

**Confirm:** JSON response with `id`, `project_id`, `name`, `description`. Note the module `id`.

### 3. Create a Task under the Module

```bash
cargo run -- task create "<module_id>" "Login Flow" "Implement login"
```

**Confirm:** JSON response includes `status: "not_started"`. Note the task `id`.

### 4. Update Task Status

```bash
cargo run -- task update "<task_id>" --status started
```

**Confirm:** JSON response shows `status: "started"`.

### 5. Append a Task Update (runtime learning)

```bash
cargo run -- task append-update "<task_id>" "Discovered that OAuth tokens expire after 1h"
```

**Confirm:** JSON response with `task_id`, `content`, and `created_at_ms` fields.

### 6. Edit a Task Update

```bash
cargo run -- task update-entry "<update_id>" "OAuth tokens expire after 1h - must refresh proactively"
```

**Confirm:** JSON response shows updated `content` and a non-null `updated_at_ms`.

### 7. List Task Updates

```bash
cargo run -- task list-updates "<task_id>"
```

**Confirm:** JSON array containing the update(s) you appended.

### 8. Create a Todo Item

```bash
cargo run -- todo create "<project_id>" "Set up CI pipeline"
```

**Confirm:** JSON response with `project_id`, `content`, `status: "pending"`.

### 9. List Todos

```bash
cargo run -- todo list "<project_id>"
```

**Confirm:** JSON array containing the todo you created.

### 10. Add Knowledge Linked to the Task

```bash
cargo run -- add --category rust --type mistake -C "Forgot to refresh OAuth token" --link-to "<task_id>"
```

**Confirm:** `{"status": "ok"}` response.

### 11. Retrieve Task Context (hierarchical traversal)

```bash
cargo run -- context --task-id "<task_id>"
```

**Confirm:** JSON with `results` array containing the mistake linked in step 10. Context includes items linked at the task, module, and project levels.

### 12. Verify Structured Errors

```bash
cargo run -- context --task-id "task:nonexistent"
```

**Confirm:** JSON error response with `status: "error"` and a meaningful message (not a stack trace).
