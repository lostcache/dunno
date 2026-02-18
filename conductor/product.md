# Initial Concept

I want to build a simple cli tool that an AI model will always consult before writing code. the cli tool will fetch
1) mistakes to avoid
2) code_styleguide
3) relevant skills
to put in the llm's context for it to use

this get's rid of having to manage markdown files within the git repo and keep it clean

# Product Guide

## Target Users
- **AI Coding Assistants:** The primary consumer, designed to integrate with tools like Cursor, Windsurf, and Copilot.
- **Human Developers:** A secondary audience, using the tool for reference and onboarding to project standards.

## Data Storage & Architecture
- **Knowledge Base:** The system will use a graph-first architecture centered on SurrealDB, with optional vector support for future semantic expansion.
- **Why:** Agents need deterministic, auditable retrieval for task execution, which is best supported by explicit hierarchy and relation edges.
- **Hierarchy:** `Project -> Module -> Task` with project-level todo queue and task-linked guidance.

## Core Functionality
- **Retrieval Interface:** The primary interaction model is a natural language query via the CLI.
    - Command: `lazydev task context --task-id <id>`
    - Output: JSON containing task-local and inherited project-global context (mistakes, style rules, skills, updates).
- **Knowledge Management (MVP):**
    - Adding/Updating Data: CLI supports creating projects/modules/tasks, linking guidance, and appending task updates.
    - Todo Queue: CLI supports project-level todo create/list/claim/complete so agents can pick the next task.
    - Runtime Learning: Agents can dynamically append mistakes they made (code/logical), including module-specific mistakes, for future retrieval.

## Compatibility
- **Initial Targets:**
    - Cursor (via context features)
    - VS Code Copilot Chat
    - Standalone LLM CLIs (e.g., Ollama, ChatGPT CLI)
