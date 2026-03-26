# Dunno Knowledge System

## Overview

`dn` is a Rust CLI tool that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike traditional natural language search, dn uses a strict graph hierarchy where context is linked to nodes and inherited down the tree.

## Command Policy

- **ALWAYS** use the `dn` binary directly. It is the only approved interface for project operations.
- **ALWAYS** run the `dn` command from the project root.
- **ONLY** use the exact `dn` subcommands documented in this file. Do NOT invent or guess subcommands.
- If a step requires a `--project-id` you don't have, first run `dn project list` to get it.
- When in doubt, use `--help` on the parent command and only use subcommands that appear in the output. Never chain made-up fallbacks.

---

## Core Hierarchy

dn organizes knowledge in a graph with two parallel structural paths:

### Code Structure Path

```
Project -> Module -> Module -> ... -> File/s (path)
```

Modules nest recursively to any depth. A child module is created with `--parent-module-id`.

### Entities

Knowledge can be attached to any structural node:

- **Project**: High-level knowledge applicable to entire codebase
- **Module**: Knowledge specific to a functional area (e.g., auth, api, utils) or nested component (e.g., auth/jwt). Modules can contain child modules.
- **File**: Knowledge tied to specific files
- **Task**: Knowledge related to specific work items
- **User Story**: Knowledge related to specific user stories
- **Epic**: Knowledge related to specific epics
- **Todo**: Knowledge related to specific todo items
- **Persona**: AI agent persona definitions linked to a project
- **Workflow**: Workflow definitions linked to a project

### Edges

- `contains` - Parent contains child (project->module, module->module, module->file)
- `has_task` - Parent has task (project->task, user_story->task, epic->task)
- `has_context` - Node has knowledge (project, module, task, epic, file -> context)
- `belongs_to_project` - Child belongs to project (task, context, user_story, epic, file, module, persona, workflow -> project)
- `belongs_to_module` - Child belongs to parent module (task, context, file, module -> module)
- `belongs_to_story` - Child belongs to user story (task -> user_story)
- `belongs_to_user_story` - Child belongs to user story (module -> user_story)
- `belongs_to_epic` - Child belongs to epic (user_story, task -> epic)
- `has_user_story` - Parent has user story (project, epic -> user_story)
- `has_epic` - Parent has epic (project -> epic)
- `has_todo` - Parent has todo (project -> todo_item)
- `has_persona` - Parent has persona (project -> persona)
- `has_workflow` - Parent has workflow (project -> workflow)

---

## Initializing the project

### Initializing for new project

1. Create a new project
   - `dn project create "ProjectName" "Description"`
2. Set up initial modules and directories as needed
   - `dn module create --project-ids <project_id> "ModuleName" "Description"`

### Initializing for existing project

1. Create a new project
   - `dn project create "ProjectName" "Description"`
2. Read the codebase for all the modules and nested modules, create them and link to the respective project and parent module.
   - `dn module add --project-ids <project_id> "ModuleName" "Description"`
   - `dn module add --project-ids <project_id> --parent-module-id <parent_module_id> "ChildModuleName" "Description"`
3. Create the file nodes with description and link to the respective project (required) and module (optional).
   - `dn file add --project-ids <project_id> --parent-ids <module_id> "FileName" "Path" "Description"`
   - `dn file add --project-ids <project_id> "FileName" "Path" "Description"` (no module)

## Initializing a task

1. Set a todo to `active` when you begin planning the associated work: `dn todo update <todo_id> --status active`
1. Fetch an item from the todo list or user-story to work on.
   - `dn todo list --project-id <project_id>`
   - `dn user-story list --project-id <project_id>`
1. If you have access to shell tool in Plan Mode switch to Plan Mode (if available) or remain in Agent Mode, but **DO NOT** create the task yet.
1. **MANDATORY:** use the `dn ctx --general -p <project>` commmand to get the project structure.
1. **MANDATORY:** Do complete research on the task. This includes:
   - Identifying the specific files that need to be modified or created.
   - Understanding the necessary schema or logic changes.
   - Formulating a step-by-step implementation plan.
1. **MANDATORY:** Present your research and implementation plan to the user and ask for
   their approval **to create the task** (not to implement it).
   - _CRITICAL:_ Even if the user explicitly says "create a task for X", you MUST present your research and get approval first. NEVER run the `dn task add` command without
     explicit user confirmation of your plan.
1. **After explicit approval**, create a task node and link to the relevant project, module and files.
   - `dn task add --project-id <project_id> "Task Name" "<THE_ENTIRE_MULTILINE_APPROVED_PLAN_VERBATIM>"`
   - _CRITICAL:_ Do NOT summarize the plan. You MUST pass the full, multi-line implementation plan that was approved by the user as the description argument.
   - _CRITICAL:_ "Making a task" or "creating a task" means running `dn task add` ONLY.
     It does NOT mean implementing the feature. After creating the task, stop — do not
     touch any code, files, or run any implementation commands unless the user explicitly
     asks you to work on the task.
   - **MANDATORY follow-up:** `dn task add` does NOT support `--file-ids`. You MUST separately link each relevant file using:
     `dn link --from-id <file_id> --edge belongs_to_task --to-ids <task_id>`
   - Do not consider the task fully created until all relevant files are linked.
1. Mark the todo `completed`: `dn todo update <todo_id> --status completed` after the task is created.

Task status values: `pending` (default on creation), `active`, `completed`.

## Starting Work

When asked to "fetch a task" or "work on a task":

1. **Always check for existing tasks first**: `dn task list --project-id <project_id>`
2. If existing tasks are found, proceed to **Working on a task**.
3. Only go to **Initializing a task** (from todos/user stories) if no existing tasks are pending.

## Working on a task

1. Before working on a task, query the context with `dn ctx --task-id <task:id> --full` to see the inherited context.
2. **MANDATORY:** If the context includes a `persona`, you MUST fully adopt it for the entire task — tone, verbosity, tool usage rules, response style, and all behavioural instructions. The persona overrides your defaults.
3. If the context includes a `workflow`, follow it exactly.
4. Mark the task as in progress.
5. **MANDATORY**: After completing the task, if the code architecture requires updating the database, do so using `dn` cli tool wihtout fail.
6. When done, mark it as completed.

## Working on an Issue

1. List issues to find what needs attention.
   - `dn issue ls` — all issues
   - `dn issue ls --task-id <task_id>` — issues for a specific task
2. Mark the issue active when you begin planning.
   - `dn issue update <issue_id> --status active`
3. Update the issue after you have a plan.
   - `dn issue update <issue_id> --plan "<updated plan>"`
4. Mark the issue completed when resolved.
   - `dn issue update <issue_id> --status completed`

Issue status values: `pending` (default on creation), `active`, `completed`.

## Context/Knowledge capture during working on a task

1. while working on a task capture the knowledge/context and link to the relevant and appropriate node task, module, project or file.
   - `dn add --field type --value <type> --field content --value "<content>" --link-to <node_id>`

### Learning Extraction

When discovering non-obvious and non-recorded information, add it to dn with appropriate type and link it to appropriate `package/module` node.

#### **What counts as a learning:**

- Hidden relationships between files/modules
- Execution paths that differ from appearance
- Non-obvious configuration, env vars, or flags
- Debugging breakthroughs when errors were misleading
- API/tool quirks and workarounds
- Build/test commands not in README
- Architectural decisions and constraints
- Files that must change together

#### **What NOT to include:**

- Obvious facts from documentation
- Standard language/framework behavior
- Things already documented
- Verbose explanations
- Session-specific details
