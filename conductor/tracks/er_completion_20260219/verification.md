# Verification Plan: ER Model Completion (Subtask & SecurityDetail)

## Automated Tests
- [ ] `cargo test --all` — all unit and integration tests pass
- [ ] `cargo fmt --all --check` — formatting clean
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — no warnings

## Manual Verification

### Subtask CRUD
```bash
# Create project, module, task
dunno project create "Test" "Test project"
dunno module create project:<id> "Core" "Core module"
dunno task create module:<id> "Parent Task" "A task with subtasks"

# Create subtasks under the task
dunno subtask create task:<id> "Subtask A" "First subtask"
dunno subtask create task:<id> "Subtask B" "Second subtask"

# List subtasks
dunno subtask list --task-id task:<id>

# Verify context includes parent chain
dunno context --subtask-id subtask:<id>
```

### SecurityDetail CRUD
```bash
# Add a security detail linked to a task
dunno add --category auth --type security --content "Validate JWT expiry before accepting tokens" --link-to task:<id>

# Retrieve task context and verify security detail appears
dunno context --task-id task:<id>
```

### Context Inheritance Verification
```bash
# Link a security detail to a project
dunno add --category data --type security --content "PII must be encrypted at rest" --link-to project:<id>

# Verify it appears in task context (inherited from project)
dunno context --task-id task:<id>
# Should contain both the task-linked and project-linked security details
```
