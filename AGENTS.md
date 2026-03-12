# AI Agent & Developer Complete Guide

Complete reference for AI agents and developers working with dn knowledge management and agent behavior standards.

---

### Core Principles

- **ALWAYS USE PARALLEL TOOLS** when applicable
- **Prefer automation**: execute actions without confirmation unless blocked by missing info or safety/irreversibility
- **Ask clarifying questions**: If blocked by missing information or when action is safety-critical/irreversible
- **No AI-generated walls of text**: Write short, focused descriptions. If you can't explain it briefly, your response might be too large
- **Understand your changes**: You must understand why your changes work. If you don't understand, say so explicitly
- **Be specific**: Avoid generic messages like "improved agent experience" - explain exactly what changed from a user perspective

---

### Coding Standards

#### General Principles

- Keep things in one function unless composable or reusable
- Prefer single word variable names where possible
- Rely on type inference; avoid explicit type annotations unless necessary for exports or clarity

#### Naming Conventions

**Mandatory for agent-written code:**

- Use single word names by default for locals, params, and helper functions
- Multi-word names only when single word would be unclear
- Do not introduce camelCase compounds when short single-word alternatives are clear
- Review touched lines before finishing; shorten newly introduced identifiers

Good names to prefer: `pid`, `cfg`, `err`, `opts`, `dir`, `root`, `child`, `state`, `timeout`

Avoid unless necessary: `inputPID`, `existingClient`, `connectTimeout`, `workerPath`

#### Code Quality (Anti-Slop)

Remove AI-generated slop:

- Extra comments a human wouldn't add
- Style inconsistent with the file
- Unnecessary emoji usage

---

### Testing

- Avoid mocks as much as possible
- Test actual implementation; do not duplicate logic into tests
- Run tests from package directories, not repo root

---

### Git & Version Control

#### Commit Messages

**Always use a prefix:**

- `docs:` - documentation changes
- `ignore:` - ignore file changes
- `wip:` - work in progress
- `{package/module/submodule}` - respective package/module/submodule

**Message content:**

- Explain **WHY** something was done from an end user perspective, not **WHAT** was done
- Be specific about user-facing changes
- If there are conflicts, DO NOT FIX THEM - notify the user instead

#### Pull Requests

**Requirements:**

- All PRs must reference an existing issue use `Ref #123` if wip and `Closes #123` if ready to close the issue.
- Keep PRs small and focused
- Explain the issue and why your change fixes it
- Before adding functionality, ensure it doesn't already exist elsewhere
- **Do not include AI-generated walls of text** - they will be ignored or closed

**Verification:**

For non-UI changes, explain:
- What did you test?
- How can a reviewer reproduce/confirm the fix?

**PR Title Format:**

Follow conventional commits:
- `feat:` - new feature or functionality
- `fix:` - bug fix
- `docs:` - documentation changes
- `chore:` - maintenance, dependencies
- `refactor:` - code refactoring without behavior change
- `test:` - adding or updating tests

Include scope when helpful: `feat({package/module/submodule}):`, `fix({package/module/submodule}):`, `chore({package/module/submodule}):`

---

### Documentation Style

- Expert technical writer tone
- Not verbose; relaxed and friendly
- Title: word or 2-3 word phrase
- Description: one short line (5-10 words), should not start with "The", avoid repeating title
- Text chunks: max 2 sentences
- Sections: separated by `---`
- Section titles: short, only first letter capitalized, imperative mood

---

### Learning Extraction

When discovering non-obvious and non-recorded information, add it to dn with appropriate type and link it to appropriate `package/module/submodule` node.

**What counts as a learning:**
- Hidden relationships between files/modules
- Execution paths that differ from appearance
- Non-obvious configuration, env vars, or flags
- Debugging breakthroughs when errors were misleading
- API/tool quirks and workarounds
- Build/test commands not in README
- Architectural decisions and constraints
- Files that must change together

**What NOT to include:**
- Obvious facts from documentation
- Standard language/framework behavior
- Things already documented
- Verbose explanations
- Session-specific details

**Keep entries to 1-3 lines per insight.**

Use dn to capture:
```bash
dn add \
  --field type --value insight \
  --field mistake|insight|some_other_field_name --value "Your learning here" \
  --field discovered_while --value "task:abc" \
  --link-to module:def
```

---

### Repository Conventions

- Default branch is `main`

---

## Part 2: Dunno Knowledge System

### Overview

`dn` is a Rust CLI tool that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike traditional natural language search, dn uses a strict graph hierarchy where context is linked to nodes and inherited down the tree.

**Key Philosophy**: Store knowledge once, retrieve it contextually based on where you are in the project hierarchy.

**AI Agent Rule**: Always query dn context before implementing. Always capture learnings after completing work.

---

### Core Hierarchy

dn organizes knowledge in a graph with two parallel structural paths:

#### Code Structure Path
```
Project -> Module -> Submodule (optional) -> File/s (path)
```

#### Work Tracking Path
```
Project -> Module -> Task
```

#### Knowledge Entities
Knowledge can be attached to any structural node:
- **Project**: High-level knowledge applicable to entire codebase
- **Module**: Knowledge specific to a functional area
- **Submodule**: Knowledge for sub-components
- **File**: Knowledge tied to specific files
- **Task**: Knowledge related to specific work items

**Note**: Context retrieval is direct-only (not inherited). Query the specific node you need context for.

---

### Core Commands for AI Agents

#### 1. Creating the Project Hierarchy

Always start by establishing the hierarchy before adding knowledge.

```bash
# Create a project (returns JSON with id like "project:abc")
dn project add "MyProject" "Description of the project"

# Create a module within a project using project ID
dn module add --project-ids project:abc "Auth" "Authentication system"

# Create a module using project name (alternative)
dn module add --project "MyProject" "Auth" "Authentication system"
# Returns: {"id":"module:def",...}

# Create a submodule (link with --module-ids)
dn submodule add --module-ids module:def "OAuth" "OAuth2 implementation"
# Returns: {"id":"submodule:ghi",...}

# Register a file (with optional description)
dn file add --parent-ids module:def "auth.rs" "src/auth.rs" "Authentication module entry point"
# Returns: {"id":"file:jkl",...}

# Register a file without description
dn file add --parent-ids module:def "utils.rs" "src/utils.rs"

# Create a task using IDs (requires both --module-ids and --project-ids, or neither for freestanding)
dn task add --module-ids module:def --project-ids project:abc "Implement JWT" "Add JWT authentication"
# Returns: {"id":"task:mno",...}

# Create a task using project name (alternative)
dn task add --module-ids module:def --project "MyProject" "Implement JWT" "Add JWT authentication"

# Create a task with case-insensitive project name matching
dn task add --module-ids module:def --project "myproject" -i "Implement JWT" "Add JWT authentication"

# List modules in a project
dn module ls --project-id project:abc
dn module ls --project "MyProject"

# List submodules (by module or by project)
dn submodule ls --module-id module:def
dn submodule ls --project-id project:abc
dn submodule ls --project "MyProject"

# List files (cascading filter priority: submodule > module > project)
dn file ls --submodule-id submodule:ghi
dn file ls --module-id module:def
dn file ls --project-id project:abc
dn file ls --project "MyProject"

# List tasks in a project
dn task ls --project-id project:abc
dn task ls --project "MyProject"
```

**AI Agent Pattern**: Capture IDs from JSON output for subsequent commands. When using project names, remember that names are unique and case-sensitive by default (use `-i` for case-insensitive matching).

---

#### 2. Adding Knowledge (Schemaless)

Knowledge is stored with arbitrary key-value pairs using `--field` for names and `--value` for values.

```bash
# Basic knowledge entry
# Each --field must be paired with a --value
dn add \
  --field type --value mistake \
  --field content --value "Avoid using unwrap in production code" \
  --link-to task:mno

# Multiple custom fields
dn add \
  --field type --value security \
  --field content --value "Always validate user inputs" \
  --field severity --value high \
  --field category --value "input-validation" \
  --field cwe --value "CWE-20" \
  --link-to module:def \
  --link-to project:abc

# Link to multiple structural nodes
dn add \
  --field type --value style \
  --field content --value "Use Result types instead of panicking" \
  --field language --value rust \
  --link-to project:abc \
  --link-to module:def \
  --link-to task:mno
```

**Common Field Patterns**:
- `type`: mistake, security, style, performance, deployment, code_review, etc.
- `content`: The actual knowledge/tip/rule (follow naming conventions - use single words where possible)
- `severity`: low, medium, high, critical
- `category`: Organization tag
- `language`: Programming language context
- `framework`: Framework-specific context
- `priority`: P0, P1, P2, etc.
- `tags`: Comma-separated or array of tags

---

#### 3. Retrieving Context

Query context for a task to get the task details, related files, hierarchy, and directly linked knowledge:

```bash
# Get context for a task (returns JSON with task, files, hierarchy, and task-linked context)
dn ctx --task-id task:mno

# Get context for a file (returns file-only context)
dn ctx --file-id file:jkl

# Get context for an epic (returns epic-only context)
dn ctx --epic-id epic:stu
```

**Task Context Returns:**
- **Task** - Full task object with id, name, description, status
- **Files** - File IDs in the parent module/submodule (files the task may touch)
- **Hierarchy** - Project, module, submodule structural info
- **Contexts** - Only knowledge directly linked to this task via `--link-to task:<id>`

**AI Agent Rule**: Always run `dn ctx --task-id <id>` before implementing to see task-specific knowledge.

---

#### 4. Work Tracking

```bash
# Create a user story using project ID
dn user-story add --project-id project:abc \
  "As a user, I want to login" \
  "User authentication feature"

# Create a user story using project name (alternative)
dn user-story add --project "MyProject" \
  "As a user, I want to login" \
  "User authentication feature"

# Create an epic using project ID
dn epic add --project-id project:abc \
  "Authentication Epic" \
  "Complete authentication system implementation"

# Create an epic using project name (alternative)
dn epic add --project "MyProject" \
  "Authentication Epic" \
  "Complete authentication system implementation"

# Create todo items using project ID
dn todo add --project-ids project:abc \
  "Review security requirements"

# Create todo items using project name (alternative)
dn todo add --project "MyProject" \
  "Review security requirements"

# List items using project ID
dn user-story ls --project-id project:abc
dn epic ls --project-id project:abc
dn todo ls --project-id project:abc
dn task ls --project-id project:abc
dn module ls --project-id project:abc
dn submodule ls --project-id project:abc
dn file ls --project-id project:abc

# List items using project name (alternative)
dn user-story ls --project "MyProject"
dn epic ls --project "MyProject"
dn todo ls --project "MyProject"
dn task ls --project "MyProject"
dn module ls --project "MyProject"
dn submodule ls --project "MyProject"
dn file ls --project "MyProject"

# List with case-insensitive matching
dn todo ls --project "myproject" -i
dn task ls --project "myproject" -i

# Delete a task when no longer needed
dn task rm task:mno
```

---

#### 5. Generic Linking

For connecting existing nodes:

```bash
# Link any two nodes with a named edge
dn link --from-id task:mno --edge has_context --to-ids context:xyz
dn link --from-id project:abc --edge contains --to-ids module:def
```

**Valid Edges**:
- `contains` - Parent contains child (project->module, module->submodule, module->file, submodule->file)
- `has_task` - Parent has task (project->task, user_story->task, epic->task)
- `has_context` - Node has knowledge (project, module, submodule, task, epic, file -> context)
- `belongs_to_project` - Child belongs to project (task, context, user_story, epic, file -> project)
- `belongs_to_module` - Child belongs to module (task, context, file -> module)
- `belongs_to_submodule` - Child belongs to submodule (context, file -> submodule)
- `belongs_to_story` - Child belongs to user story (task -> user_story)
- `belongs_to_user_story` - Child belongs to user story (module, submodule -> user_story)
- `belongs_to_epic` - Child belongs to epic (user_story, task -> epic)
- `has_user_story` - Parent has user story (project, epic -> user_story)
- `has_epic` - Parent has epic (project -> epic)
- `has_todo` - Parent has todo (project -> todo_item)
- `has_module` - Parent has module (user_story -> module)
- `has_submodule` - Parent has submodule (user_story -> submodule)

---

### Best Practices for AI Agents

#### 1. Establish Hierarchy First

Always add the project structure before adding knowledge:

```bash
# Step 1: Create project and capture ID
PROJECT=$(dn project add "MyApp" "Web application" | jq -r '.id')

# Step 2: Create modules with project link
MODULE=$(dn module add --project-ids "$PROJECT" "API" "REST API" | jq -r '.id')

# Step 3: Create tasks with module and project links
TASK=$(dn task add --module-ids "$MODULE" --project-ids "$PROJECT" \
  "Implement auth" "JWT authentication" | jq -r '.id')

# Step 4: Now add knowledge linked to appropriate nodes
dn add \
  --field type --value mistake \
  --field content --value "Don't store secrets in env vars" \
  --link-to "$TASK"
```

---

#### 2. Capture and Reuse IDs

Always parse JSON responses to capture IDs for subsequent operations:

```bash
# Capture IDs for reuse
PROJECT_ID=$(dn project add "App" "Description" | jq -r '.id')
echo "Created project: $PROJECT_ID"

# Use in subsequent commands
dn module add --project-ids "$PROJECT_ID" "Core" "Core module"
```

---

#### 4. Rich Field Usage

Always add context-enriching fields:

```bash
dn add \
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

---

#### 5. Context Retrieval Workflow

When working on a task, always retrieve relevant context:

```bash
# Before implementing, get context for the task
dn ctx --task-id task:abc --pretty'

# Also check module-level context
dn ctx --file-id file:def --pretty'

# Combine and analyze all relevant knowledge
```

---

#### 6. Error Handling

All commands return structured JSON:

**Success**:
```json
{"status":"ok"}
```

**Error**:
```json
{"status":"error","kind":"runtime_error","error":"Task not found: task:123"}
```

---

### Common AI Agent Workflows

#### Workflow 1: Learning from Mistakes

```bash
# User reports an issue - capture as knowledge
MISTAKE_ID=$(dn add \
  --field type --value mistake \
  --field content --value "Race condition in async handler" \
  --field language --value rust \
  --field symptom --value "Intermittent 500 errors under load" \
  --field root_cause --value "Shared state without proper locking" \
  --field fix --value "Use Arc<Mutex<T>> for shared state" \
  --link-to task:abc | jq -r '.id // empty')

# Later, when working on similar tasks, query context
dn ctx --task-id task:abc
```

---

#### Workflow 2: Code Review Knowledge

```bash
# Store code review feedback as knowledge
REVIEW_ID=$(dn add \
  --field type --value code_review \
  --field content --value "Extract database queries into repository pattern" \
  --field severity --value medium \
  --field rationale --value "Improves testability and maintainability" \
  --field effort --value "2 hours" \
  --field pr_number --value "#42" \
  --link-to file:src/main.rs)
```

---

#### Workflow 3: Security Knowledge Management

```bash
# Log security findings with rich metadata
SECURITY_ID=$(dn add \
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

---

#### Workflow 4: Performance Optimization Knowledge

```bash
# Capture performance insights
PERF_ID=$(dn add \
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

---

#### Workflow 5: AI Agent Task Planning and Execution

Complete workflow for planning and executing tasks with proper context retrieval:

```bash
# Step 1: Create a todo to track the work item
# Step 2: Query the task to check if already exists and whether to add as a subtask.
# Step 3: Query the knowledge base for code structure and existing context
# Step 4: Create a task linked to the appropriate module/submodule/epic
# Step 5: Update task with a detailed plan as knowledge

# Step 6: Query knowledge base again for relevant patterns before implementation
# Get context specific to the task
dn ctx --task-id "$TASK_ID" | jq '.results[]'

# Step 7: After completion, capture insights and lessons learned
dn add \
  --field type --value insight \
  --field content --value "JWT tokens should have short expiry with refresh token pattern" \
  --field related_task --value "$TASK_ID" \
  --link-to "$TASK_ID" \
  --link-to module:def
```

**AI Agent Best Practices for Task Planning**:

1. **Always query before planning**: Retrieve existing context from project, module, and relevant files
2. **Create todos first**: Track work items before creating detailed tasks
3. **Link strategically**: Attach tasks to the most specific module/submodule and relevant epics
4. **Always link to files/modules**: After adding a task, link it to relevant files and modules it touches
5. **Store plans as knowledge**: Use `--field type --value plan` to document implementation approach
6. **Query task context before coding**: Always run `dn ctx --task-id <id>` before implementation
7. **Capture learnings**: After task completion, add insights linked to the task and relevant modules

**Example: Full Planning Session**:

```bash
# 1. Initial query - understand existing structure
dn ctx --project-id project:abc | jq '.results[] | {type, content}'

# 2. Create tracking todo
TODO=$(dn todo add --project-ids project:abc "Add OAuth integration" | jq -r '.id')

# 3. Explore specific module context
dn ctx --module-id module:auth | jq '.results[]'

# 4. Create and link task
TASK=$(dn task add \
  --module-ids module:auth \
  --project-ids project:abc \
  "Implement OAuth2 flow" \
  "Add OAuth2 authentication with Google and GitHub providers" | jq -r '.id')

# 4b. Link task to relevant files
# dn link --from "$TASK" --edge has_file --to file:auth.rs

# 5. Document the plan
dn add \
  --field type --value plan \
  --field content --value "OAuth2 implementation strategy" \
  --field approach --value "Use OAuth2 crate with state parameter for CSRF protection" \
  --field providers --value "Google, GitHub" \
  --field callback_url --value "/auth/callback" \
  --link-to "$TASK"

# 6. Verify context is properly linked
dn ctx --task-id "$TASK" | jq '.results | length'
```

---

### Integration with AI Workflows

#### Context Injection

Before generating code, retrieve and inject relevant context:

```python
import subprocess
import json

def get_task_context(task_id):
    result = subprocess.run(
        ["dn", "ctx", "--task-id", task_id],
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

---

#### Automated Knowledge Capture

After completing a task, automatically capture insights:

```python
def capture_knowledge(task_id, content, knowledge_type="insight", **fields):
    cmd = ["dn", "add", "--field", "type", "--value", knowledge_type]
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

---

## Part 3: Troubleshooting & Reference

### Troubleshooting Dunno

#### Common Issues

1. **"Number of --field flags must match --value flags"**
   - Ensure every `--field` has a corresponding `--value`
   - Count must be exactly equal

2. **"Task not found" / "Module not found"**
   - Verify IDs are correct format (e.g., `task:abc`, `module:def`)
   - Check that the node exists with `dn task ls` or similar

3. **"At least one --field key=value pair is required"**
   - The `add` command requires at least one pair of `--field` and `--value`

4. **Database connection errors**
   - For local: Check write permissions to `~/.local/share/dn/`
   - For cloud: Verify credentials and network connectivity

---

#### Configuration Debugging

```bash
# Check resolved configuration
dn config show

# Test with explicit backend
dn --backend local add --field type --value test --field content --value "test"
```

---

#### Purging Data (Development Only)

```bash
# ⚠️ DANGER: Delete all data (irreversible)
dn purge
```

---

### Tips for Maximum Effectiveness

1. **Be specific in content**: "Use Result types" is better than "Handle errors"
2. **Add context**: Always include why something is important
3. **Link strategically**: Attach knowledge to the most specific relevant node
4. **Use consistent vocabulary**: Standardize type names (mistake, security, style)
5. **Keep it actionable**: Content should guide future decisions
6. **Version your knowledge**: Add `date`, `version`, or `commit` fields for temporal context
7. **Cross-reference**: Use `--link-to` to connect knowledge to multiple relevant nodes
8. **Query before acting**: Always check `dn ctx` before implementing

---

### Advanced Patterns

#### Knowledge Templates

Create reusable knowledge structures:

```bash
# Security vulnerability template
dn add \
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

---

#### Knowledge Queries

Use jq for sophisticated filtering:

```bash
# Find all high-security knowledge
dn ctx --task-id task:abc | jq '.results[] | select(.severity == "high" and .type == "security")'

# Find mistakes in specific language
dn ctx --module-id module:def | jq '.results[] | select(.type == "mistake" and .language == "rust")'

# Find unreviewed knowledge (no pr_number field)
dn ctx --project-id project:abc | jq '.results[] | select(has("pr_number") | not)'
```

---

#### Batch Operations

Process multiple knowledge entries:

```bash
# Create a file with knowledge entries
while read -r line; do
  IFS='|' read -r type content module <<< "$line"
  dn add \
    --field type --value "$type" \
    --field content --value "$content" \
    --link-to "$module"
done < knowledge_list.txt
```

---

## Summary for AI Agents

**Before Starting Work:**
1. Query dn context for your task: `dn ctx --task-id <id>`
2. Review captured learnings and pitfalls
3. Follow naming conventions (single words)

**During Work:**
1. Use parallel tools when possible
2. Keep responses concise (1-3 sentences)
3. Understand why your changes work
4. No AI-generated walls of text

**After Completing Work:**
1. Capture learnings: `dn add --field type --value insight`
2. Link to task and relevant modules
3. Keep entries to 1-3 lines
4. Use specific, actionable content

**Remember**: The power of dn lies in its deterministic, hierarchical context retrieval. Always structure your knowledge to match your project hierarchy, and query context before making decisions.

---

**Note**: This codebase is documented in dn under project ID `project:nx7h5j92o078xa4pmo1y` with modules for CLI, Config, Database, Context, Ingest, and Models.
