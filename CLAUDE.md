# Dunno Knowledge System

## Overview

`dn` is a Rust CLI tool that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike traditional natural language search, dn uses a strict graph hierarchy where context is linked to nodes and inherited down the tree.

## Command Policy

- **MANDATORY:** Use the `dn` binary from the project root.
- **ALWAYS** use the `dn` binary directly. It is the only approved interface for project operations.
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
- **Issue**: Bug or a problem, optionally linked to a task

#### All entities have an id of the form `<entity>:<id>` never use just the `<id>` as the id.

### Edges

- `contains` - Parent contains child (project->module, module->file)
- `has_module` - Parent module has child module (module->module)
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
- `has_issue` - Parent has issue (task -> issue)
- `belongs_to_task` - Child belongs to task (issue -> task)

---

## Initializing the project

### Initializing for new project

1. Create a new project
   - `dn project add "ProjectName" "Description"`
2. Set up initial modules and directories as needed
   - `dn module add --project-id <project_id> --name "ModuleName" --desc "Description" --pmid ""`
   - Repeat `--name`/`--desc`/`--pmid` to create multiple modules in one command.

### Initializing for existing project

1. Create a new project
   - `dn project create "ProjectName" "Description"`
2. Read the codebase for all the modules and nested modules, create them and link to the respective project and parent module.
   - `dn module add --project-id <project_id> --name "ModuleName" --desc "Description" --pmid ""`
   - `dn module add --project-id <project_id> --name "Child" --desc "Description" --pmid <parent_module_id>`
   - To create several modules in one call, repeat `--name`/`--desc`/`--pmid` together; counts must match (pass `--pmid ""` for top-level modules).
3. Create the file nodes with description and link to the respective project (required) and module (optional).
   - `dn file add --project-id <project_id> --name "FileName" --path "Path" --description "Description" --parent-mod-id <module_id>`
   - `dn file add --project-id <project_id> --name "FileName" --path "Path" --description "Description" --parent-mod-id ""` (no module)
   - To create several files in one call, repeat `--name`/`--path`/`--description`/`--parent-mod-id` together; counts must match (pass `--parent-mod-id ""` for freestanding files).

> **Repeatable flags:** `module add` and `file add` create one entity per group of repeated flags, all in a single transaction. All repeatable flag counts must be equal (enforced by the handler). A single `--project-id` (or `-p`/`--project` name) applies to every entity in the batch.

## Initializing a task

1. If given a todo/user-story id fetch using `dn {todo/user-story} get <id> ` else an item from the todo list or user-story to work on.
   - `dn todo list --project-id <project_id>`
   - `dn user-story list --project-id <project_id>`
1. Set a todo to `active` when you begin planning the associated work: `dn todo update <todo_id> --status active`
1. If you have access to shell tool in Plan Mode switch to Plan Mode (if available) or remain in Agent Mode, but **DO NOT** create the task yet.
1. **MANDATORY:** use the `dn ctx --general -p <project>` commmand to get the project structure. It is the PRIMARY source for understanding the codebase structure.
   Every exploation attempt after his should be intentional and educated.
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
     - Link single file with `dn link --from-id <file_id> --edge belongs_to_task --to-id <task_id>`.
     - You may repeat the flags to link multiple files in the same command
       `dn link --from-id <file_id> --edge belongs_to_task --to-id <task_id> --from-id <file_id> --edge belongs_to_task --to-id <task_id>`.
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
5. **MANDATORY**: After completing the task, run all the tests and make sure they pass.
6. **MANDATORY**: After completing the task, if the code architecture requires updating the database, do so using `dn` cli tool wihtout fail.
7. When done, mark it as completed.

## Reviewing a task to add an issue

1. Only review tasks that are comleted. If the task is pendng or in progress just report, do not review it.
2. Retrive the task context with `dn ctx --task-id <task:id> --full` to see the inherited context.
3. Review all the code changes for code smells, security issues, and other issues.
4. Report the finding to the use asking whether to create an issue or not.
5. If prompted to create an issue, create it using `dn issue add --project-id <PROJECT_ID> <SHORT_DESCRIPTION> --task-id <TASK_ID> --plan <PLAN>`.

## planning for an user added issue

1. **MANDATORY**: If given an issue id fetch using `dn issue get <id>` else an issue from the issue list to work on.
2. **MANDATORY**: Then get the general context to avoid fuzzy reads using `dn ctx --general -p <project_id>`.
3. **MANDATORY**: Mark the issue active when you begin planning if it doesn't already have a plan.
4. **MANDATORY**: Ask the user whether the plan looks good before updating the issue.
5. **MANDATORY**: Update the issue after you have a plan using `dn issue update <issue_id> --plan "<plan>"`
6. **Note**: Do not work on the issue unless the user explicitly requests it.

## Working on an Issue

1. If given an issue id fetch using `dn issue get <id>` else an issue from the issue list to work on.
   - `dn issue ls --project-id <project_id>` — all issues for a project
   - `dn issue ls --project-id <project_id> --task-id <task_id>` — issues for a specific task
2. Mark the issue active to begin working on it.
   - `dn issue update <issue_id> --status active`
3. Mark the issue completed when resolved.
   - `dn issue update <issue_id> --status completed`

Issue status values: `pending` (default on creation), `active`, `completed`.

## Context/Knowledge capture during working on a task

1. while working on a task capture the knowledge/context and link to the relevant and appropriate node task, module, project or file.
   - `dn add --field type --value <type> --field content --value "<content>" --link-to <node_id>`
2. To delete a context entry:
   - `dn rm <context_id> [<context_id>...]`

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
