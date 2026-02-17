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
- **Knowledge Base:** The system will utilize a combination of a Graph Database and a Vector Database.
- **Why:** This hybrid approach ensures efficient and semantically relevant querying of "mistakes to avoid," "code style guides," and "relevant skills."

## Core Functionality
- **Retrieval Interface:** The primary interaction model is a natural language query via the CLI.
    - Command: `lazydev {query}`
    - Output: The tool will output all relevant details (mistakes, style rules, skills) needed for the context.
- **Knowledge Management (MVP):**
    - Adding/Updating Data: A simple CLI interface will be used for the MVP to manually add new knowledge entries.

## Compatibility
- **Initial Targets:**
    - Cursor (via context features)
    - VS Code Copilot Chat
    - Standalone LLM CLIs (e.g., Ollama, ChatGPT CLI)
