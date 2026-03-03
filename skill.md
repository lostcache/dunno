# Dunno Tool Skill Guide for AI Agents

## Overview

`dunno` is a Rust CLI tool that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike traditional natural language search, dunno uses a strict graph hierarchy where context is linked to nodes and inherited down the tree.

**Key Philosophy**: Store knowledge once, retrieve it contextually based on where you are in the project hierarchy.

## Core Hierarchy

dunno organizes knowledge in a graph with two parallel structural paths:

### Code Structure Path
```
Project -> Module -> Submodule (optional) -> File/s (path)
```

### Work Tracking Path
```
Project -> Module -> Task -> Subtask (optional)
```

### Knowledge Entities
Knowledge can be attached to any structural node:
- **Project**: High-level knowledge applicable to entire codebase
- **Module**: Knowledge specific to a functional area
- **Submodule**: Knowledge for sub-components
- **File**: Knowledge tied to specific files
- **Task**: Knowledge related to specific work items
- **Subtask**: Granular knowledge for sub-tasks

**Note**: Context retrieval is direct-only (not inherited). Query the specific node you need context for.

## Installation & Setup

```bash
# Build from source
cargo build --release

# Binary location
./target/release/dunno
```

### Configuration Priority (highest to lowest)
1. CLI flags (`--backend`)
2. Environment variables
3. `~/.config/dunno/config.toml`
4. Built-in defaults (local storage at `~/.local/share/dunno/data.db`)

### Environment Variables
- `DUNNO_BACKEND` - "local" or "cloud"
- `DUNNO_LOCAL_PATH` - Path for local database
- `DUNNO_CLOUD_URL` - SurrealDB Cloud URL
- `DUNNO_CLOUD_NS` - Cloud namespace
- `DUNNO_CLOUD_DB` - Cloud database name
- `DUNNO_CLOUD_USER` - Cloud username
- `DUNNO_CLOUD_PASS` - Cloud password
- `DUNNO_CLOUD_AUTH_TYPE` - "root", "namespace", or "database"

## Core Commands for AI Agents

### 1. Creating the Project Hierarchy

Always start by establishing the hierarchy before adding knowledge.

```bash
# Create a project (returns JSON with id like "project:abc")
dunno project create "MyProject" "Description of the project"

# Create a module within a project (link with --project-ids)
dunno module create --project-ids project:abc "Auth" "Authentication system"
# Returns: {"id":"module:def",...}

# Create a submodule (link with --module-ids)
dunno submodule create --module-ids module:def "OAuth" "OAuth2 implementation"
# Returns: {"id":"submodule:ghi",...}

# Register a file
dunno file create --parent-ids module:def "auth.rs" "src/auth.rs"
# Returns: {"id":"file:jkl",...}

# Create a task (requires both --module-ids and --project-ids, or neither for freestanding)
dunno task create --module-ids module:def --project-ids project:abc "Implement JWT" "Add JWT authentication"
# Returns: {"id":"task:mno",...}

# Create a subtask
dunno subtask create --task-ids task:mno "Write tests" "Unit tests for JWT"
# Returns: {"id":"subtask:pqr",...}
```

**AI Agent Pattern**: Capture IDs from JSON output for subsequent commands.

### 2. Adding Knowledge (Schemaless)

Knowledge is stored with arbitrary key-value pairs using `--field` for names and `--value` for values.

```bash
# Basic knowledge entry
# Each --field must be paired with a --value
dunno add \
  --field type --value mistake \
  --field content --value "Avoid using unwrap in production code" \
  --link-to task:mno

# Multiple custom fields
dunno add \
  --field type --value security \
  --field content --value "Always validate user inputs" \
  --field severity --value high \
  --field category --value "input-validation" \
  --field cwe --value "CWE-20" \
  --link-to module:def \
  --link-to project:abc

# Link to multiple structural nodes
dunno add \
  --field type --value style \
  --field content --value "Use Result types instead of panicking" \
  --field language --value rust \
  --link-to project:abc \
  --link-to module:def \
  --link-to task:mno
```

**Common Field Patterns**:
- `type`: mistake, security, style, performance, deployment, code_review, etc.
- `content`: The actual knowledge/tip/rule
- `severity`: low, medium, high, critical
- `category`: Organization tag
- `language`: Programming language context
- `framework`: Framework-specific context
- `priority`: P0, P1, P2, etc.
- `tags`: Comma-separated or array of tags

### 3. Retrieving Context

Query context directly linked to a specific node:

```bash
# Get context for a task (returns JSON array)
dunno context --task-id task:mno

# Get context for a file
dunno context --file-id file:jkl

# Get context for a subtask
dunno context --subtask-id subtask:pqr

# Get context for an epic
dunno context --epic-id epic:stu
```

**Response Format**:
```json
{
  "results": [
    {
      "id": "context:xyz",
      "type": "mistake",
      "content": "Avoid using unwrap in production code",
      "severity": "high",
      "category": "error-handling",
      "node_type": "mistake"
    }
  ]
}
```

### 4. Work Tracking (Optional)

```bash
# Create a user story
dunno user-story create --project-id project:abc \
  "As a user, I want to login" \
  "User authentication feature"

# Create an epic
dunno epic create --project-id project:abc \
  "Authentication Epic" \
  "Complete authentication system implementation"

# Create todo items
dunno todo create --project-ids project:abc \
  "Review security requirements"

# List items
dunno user-story list --project-id project:abc
dunno epic list --project-id project:abc
dunno todo list --project-id project:abc

# Delete a task when no longer needed
dunno task delete task:mno
```

### 5. Generic Linking

For connecting existing nodes:

```bash
# Link any two nodes with a named edge
dunno link --from-id task:mno --edge has_context --to-ids context:xyz
dunno link --from-id project:abc --edge contains --to-ids module:def
```

**Valid Edges**:
- `contains` - Parent contains child (project->module, module->submodule, etc.)
- `has_task` - Parent has task (project->task, user_story->task, epic->task)
- `has_context` - Node has knowledge (any structural node -> context)
- `belongs_to_project` - Child belongs to project
- `belongs_to_module` - Child belongs to module
- `belongs_to_task` - Child belongs to task
- `belongs_to_user_story` - Child belongs to user story
- `belongs_to_epic` - Child belongs to epic
- `has_user_story` - Parent has user story
- `has_epic` - Parent has epic
- `has_todo` - Parent has todo
- `has_subtask` - Task has subtask
- `has_module` - User story has module
- `has_submodule` - User story has submodule

## Best Practices for AI Agents

### 1. Establish Hierarchy First

Always create the project structure before adding knowledge:

```bash
# Step 1: Create project and capture ID
PROJECT=$(dunno project create "MyApp" "Web application" | jq -r '.id')

# Step 2: Create modules with project link
MODULE=$(dunno module create --project-ids "$PROJECT" "API" "REST API" | jq -r '.id')

# Step 3: Create tasks with module and project links
TASK=$(dunno task create --module-ids "$MODULE" --project-ids "$PROJECT" \
  "Implement auth" "JWT authentication" | jq -r '.id')

# Step 4: Now add knowledge linked to appropriate nodes
dunno add \
  --field type --value mistake \
  --field content --value "Don't store secrets in env vars" \
  --link-to "$TASK"
```

### 2. Capture and Reuse IDs

Always parse JSON responses to capture IDs for subsequent operations:

```bash
# Capture IDs for reuse
PROJECT_ID=$(dunno project create "App" "Description" | jq -r '.id')
echo "Created project: $PROJECT_ID"

# Use in subsequent commands
dunno module create --project-ids "$PROJECT_ID" "Core" "Core module"
```

### 3. Knowledge Granularity

- **Project level**: High-level architectural decisions, tech stack choices
- **Module level**: Domain-specific patterns, API contracts
- **Task level**: Implementation details, specific pitfalls
- **File level**: File-specific conventions, naming patterns

### 4. Rich Field Usage

Always add context-enriching fields:

```bash
dunno add \
  --field type --value security \
  --field content --value "SQL injection vulnerability in user input" \
  --field severity --value critical \
  --field cwe --value "CWE-89" \
  --field owasp --value "A03:2021-Injection" \
  --field remediation --value "Use parameterized queries" \
  --field example_bad --value "query = 'SELECT * FROM users WHERE id = ' + userId" \
  --field example_good --value "query = 'SELECT * FROM users WHERE id = ?'; db.query(query, [userId])" \
  --link-to task:abc
```

### 5. Context Retrieval Workflow

When working on a task, always retrieve relevant context:

```bash
# Before implementing, get context for the task
dunno context --task-id task:abc | jq '.results[]'

# Also check module-level context
dunno context --file-id file:def | jq '.results[]'

# Combine and analyze all relevant knowledge
```

### 6. Error Handling

All commands return structured JSON:

**Success**:
```json
{"status":"ok"}
```

**Error**:
```json
{"status":"error","kind":"runtime_error","error":"Task not found: task:123"}
```

**AI Agent Pattern**: Always check `status` field in responses.

## Common AI Agent Workflows

### Workflow 1: Learning from Mistakes

```bash
# User reports an issue - capture as knowledge
MISTAKE_ID=$(dunno add \
  --field type --value mistake \
  --field content --value "Race condition in async handler" \
  --field language --value rust \
  --field symptom --value "Intermittent 500 errors under load" \
  --field root_cause --value "Shared state without proper locking" \
  --field fix --value "Use Arc<Mutex<T>> for shared state" \
  --link-to task:abc | jq -r '.id // empty')

# Later, when working on similar tasks, query context
dunno context --task-id task:abc | jq '.results[] | select(.type == "mistake")'
```

### Workflow 2: Code Review Knowledge

```bash
# Store code review feedback as knowledge
REVIEW_ID=$(dunno add \
  --field type --value code_review \
  --field content --value "Extract database queries into repository pattern" \
  --field severity --value medium \
  --field rationale --value "Improves testability and maintainability" \
  --field effort --value "2 hours" \
  --field pr_number --value "#42" \
  --link-to file:src/main.rs)
```

### Workflow 3: Security Knowledge Management

```bash
# Log security findings with rich metadata
SECURITY_ID=$(dunno add \
  --field type --value security \
  --field content --value "Hardcoded API key in config file" \
  --field severity --value critical \
  --field cwe --value "CWE-798" \
  --field cve --value "CVE-2023-1234" \
  --field scan_tool --value "truffleHog" \
  --field file --value "config/secrets.yml" \
  --field line --value "15" \
  --field remediation --value "Move to environment variables or secret manager" \
  --link-to module:api \
  --link-to project:main)
```

### Workflow 4: Performance Optimization Knowledge

```bash
# Capture performance insights
PERF_ID=$(dunno add \
  --field type --value performance \
  --field content --value "N+1 query problem in user listing" \
  --field severity --value high \
  --field metric_before --value "2500ms response time" \
  --field metric_after --value "120ms response time" \
  --field solution --value "Use eager loading with JOIN" \
  --field benchmark --value "ab -n 1000 -c 10" \
  --field improvement --value "20x faster" \
  --link-to module:users)
```

## Integration with AI Workflows

### Context Injection

Before generating code, retrieve and inject relevant context:

```python
import subprocess
import json

def get_task_context(task_id):
    result = subprocess.run(
        ["dunno", "context", "--task-id", task_id],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

# Use in prompt
task_id = "task:abc"
context = get_task_context(task_id)
knowledge = "\n".join([
    f"- {c['type']}: {c['content']}"
    for c in context.get("results", [])
])

prompt = f"""
You are working on task {task_id}.

Relevant knowledge:
{knowledge}

Now generate the implementation...
"""
```

### Automated Knowledge Capture

After completing a task, automatically capture insights:

```python
def capture_knowledge(task_id, content, knowledge_type="insight", **fields):
    cmd = ["dunno", "add", "--field", "type", "--value", knowledge_type]
    cmd.extend(["--field", "content", "--value", content])
    cmd.extend(["--field", "source_task", "--value", task_id])
    
    for key, value in fields.items():
        cmd.extend(["--field", key, "--value", str(value)])
    cmd.extend(["--link-to", task_id])
    
    subprocess.run(cmd, capture_output=True)

# Usage
capture_knowledge(
    task_id="task:abc",
    content="Found that caching TTL should be 5 minutes for this endpoint",
    knowledge_type="discovery",
    category="caching",
    endpoint="/api/users"
)
```

## Troubleshooting

### Common Issues

1. **"Number of --field flags must match --value flags"**
   - Ensure every `--field` has a corresponding `--value`
   - Count must be exactly equal

2. **"Task not found" / "Module not found"**
   - Verify IDs are correct format (e.g., `task:abc`, `module:def`)
   - Check that the node exists with `dunno task list` or similar

3. **"At least one --field key=value pair is required"**
   - The `add` command requires at least one pair of `--field` and `--value`

4. **Database connection errors**
   - For local: Check write permissions to `~/.local/share/dunno/`
   - For cloud: Verify credentials and network connectivity

### Configuration Debugging

```bash
# Check resolved configuration
dunno config show

# Test with explicit backend
dunno --backend local add --field type --value test --field content --value "test"
```

### Purging Data (Development Only)

```bash
# ⚠️ DANGER: Delete all data (irreversible)
dunno purge
```

## Tips for Maximum Effectiveness

1. **Be specific in content**: "Use Result types" is better than "Handle errors"
2. **Add context**: Always include why something is important
3. **Link strategically**: Attach knowledge to the most specific relevant node
4. **Use consistent vocabulary**: Standardize type names (mistake, security, style)
5. **Keep it actionable**: Content should guide future decisions
6. **Version your knowledge**: Add `date`, `version`, or `commit` fields for temporal context
7. **Cross-reference**: Use `--link-to` to connect knowledge to multiple relevant nodes
8. **Query before acting**: Always check `dunno context` before implementing

## Advanced Patterns

### Knowledge Templates

Create reusable knowledge structures:

```bash
# Security vulnerability template
dunno add \
  --field type --value security \
  --field content --value "[VULNERABILITY NAME]" \
  --field severity --value "[critical|high|medium|low]" \
  --field cwe --value "[CWE-ID]" \
  --field owasp_top10 --value "[Category]" \
  --field affected_versions --value "[Version range]" \
  --field remediation --value "[Fix description]" \
  --field references --value "[CVE, advisory links]" \
  --field discovered_date --value "[ISO date]" \
  --field fixed_date --value "[ISO date or pending]" \
  --link-to [NODE_ID]
```

### Knowledge Queries

Use jq for sophisticated filtering:

```bash
# Find all high-security knowledge
dunno context --task-id task:abc | jq '.results[] | select(.severity == "high" and .type == "security")'

# Find mistakes in specific language
dunno context --module-id module:def | jq '.results[] | select(.type == "mistake" and .language == "rust")'

# Find unreviewed knowledge (no pr_number field)
dunno context --project-id project:abc | jq '.results[] | select(has("pr_number") | not)'
```

### Batch Operations

Process multiple knowledge entries:

```bash
# Create a file with knowledge entries
while read -r line; do
  IFS='|' read -r type content module <<< "$line"
  dunno add \
    --field type --value "$type" \
    --field content --value "$content" \
    --link-to "$module"
done < knowledge_list.txt
```

---

**Remember**: The power of dunno lies in its deterministic, hierarchical context retrieval. Always structure your knowledge to match your project hierarchy, and query context before making decisions.
