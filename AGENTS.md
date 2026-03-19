# Dunno Knowledge System

## Overview

`dn` is a Rust CLI tool that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike traditional natural language search, dn uses a strict graph hierarchy where context is linked to nodes and inherited down the tree.

---

## Core Hierarchy

dn organizes knowledge in a graph with two parallel structural paths:

### Code Structure Path

```
Project -> Module -> Submodule (optional) -> File/s (path)
```

### Entities

Knowledge can be attached to any structural node:

- **Project**: High-level knowledge applicable to entire codebase
- **Module**: Knowledge specific to a functional area (e.g., auth, api, utils)
- **Submodule**: Knowledge for nested components (e.g., auth/jwt, auth/session)
- **File**: Knowledge tied to specific files
- **Task**: Knowledge related to specific work items
- **User Story**: Knowledge related to specific user stories
- **Epic**: Knowledge related to specific epics
- **Todo**: Knowledge related to specific todo items

### Edges

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

## Initializing the project

### Initializing for new project

1. Create a new project
   - `dn project create "ProjectName" "Description"`
2. Set up initial modules and directories as needed
   - `dn module create --project-ids <project_id> "ModuleName" "Description"`

### Initializing for existing project

1. Create a new project
   - `dn project create "ProjectName" "Description"`
2. Read the codebase for all the modules and submodules, create them and link to the respective project and module.
   - `dn module create --project-ids <project_id> "ModuleName" "Description"`
   - `dn submodule create --module-ids <module_id> "SubmoduleName" "Description"`
3. Create the file nodes with description and link to the respective module/submodule and project.
   - `dn file create --parent-ids <module_or_submodule_id> "FileName" "Path" "Description"`

## Creating a task

1. Fetch an item from the todo list or user-story to work on.
   - `dn todo list --project-id <project_id>`
   - `dn user-story list --project-id <project_id>`
2. Enter planning mode even if currently in agent mode.
3. Do complete research of the task and consult the user.
4. After approval, create a task node and link to the relevant project, module/submodule and files.
   - `dn task create --project-ids <project_id> --module-ids <module_id> "Task Name" "Description"`

## Working on a task

1. Before working on a task, query the context with `dn ctx --task-id <id> --full` to see the inherited context.
2. Mark the task as in progress.

## Context/Knowledge capture during working on a task

1. while working on a task capture the knowledge/context and link to the relevant and appropriate node task, module/submodule, project or file.
   - `dn add --field type --value <type> --field content --value "<content>" --link-to <node_id>`

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
