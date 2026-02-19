# Product Guide

## Target Users
- **AI Coding Assistants:** The primary consumer, designed to integrate with tools like Cursor, Windsurf, and Copilot.
- **Human Developers:** A secondary audience, using the tool for reference and onboarding to project standards.

## Data Storage & Architecture
- **Knowledge Base:** The system uses a strict graph hierarchy powered by SurrealDB.
- **Why:** Agents need deterministic, auditable retrieval for task execution, best supported by explicit hierarchy and relation edges. Natural language search is replaced by precise structural traversal.
- **Hierarchy:** `Project -> Module -> Task`. Context is inherited down this path.
- **Storage Modes:**
    - **Local (default):** Embedded SurrealDB persisted to disk. Zero-config, works offline, single-binary experience. Ideal for individual developers.
    - **Cloud:** Remote SurrealDB instance (e.g., SurrealDB Cloud). Enables cross-machine sync and team-shared knowledge bases.
- **Configuration:** All storage settings live in `~/.config/dunno/config.toml`. The CLI works out of the box with no config file (local mode with defaults). Backend switching is a single config toggle.

## Core Functionality
- **Retrieval Interface:** The interaction model is ID-based traversal via the CLI.
    - Command: `lazydev task context --task-id <id>`
    - Output: JSON containing task-local context merged with inherited module and project context (mistakes, style rules, skills).
- **Knowledge Management:**
    - **Hierarchy Management:** CLI supports creating projects, modules, and tasks to build the work tree.
    - **Todo Queue:** CLI supports project-level todo management. Agents claim todo items which map to specific tasks.
    - **Context Linking:** Mistakes and rules are linked explicitly to Projects, Modules, or Tasks.
    - **Runtime Learning:** Agents can append new mistakes or rules to the current task/module for future reference.

## Compatibility
- **Initial Targets:**
    - Cursor (via context features)
    - VS Code Copilot Chat
    - Standalone LLM CLIs
