# Product Guidelines

## Tone and Style
- **Technical & Dry:** The CLI's output should be focused purely on data, prioritizing machine readability and clarity over conversational tone.
- **Minimalist:** Output only essential information needed for the AI context or the specific query.

## Error Handling
- **Structured Errors:** All errors or missing information scenarios must return a standard JSON response (e.g., `{"error": "No data found"}`). This ensures consistent parsing by consuming tools.

## Naming Conventions
- **Kebab-Case:** All CLI commands and flags must use kebab-case (e.g., `lazydev get-context --mistakes`). This aligns with standard CLI practices.

## Visual Identity
- **Purely Functional:** No ASCII art, logos, or decorative elements. Help output and non-JSON responses should be plain text to avoid cluttering logs or terminal buffers.

## Versioning
- **Semantic Versioning (SemVer):** The project will strictly adhere to SemVer (e.g., 1.0.0, 1.1.0) to ensure predictable compatibility for integrations and users.
