# Track Specification: MVP - CLI & Database Integration

## Overview
This track focuses on building the Minimum Viable Product (MVP) of the `lazydev` CLI. The goal is to create a functional tool that can accept natural language queries and return relevant coding context (mistakes, style guides, skills) by leveraging a hybrid database approach (SurrealDB + Qdrant).

## Objectives
1.  **CLI Interface:** Implement a robust CLI using `clap` with commands to `add` new knowledge and `get-context` based on queries.
2.  **Data Modeling:** Define structured data models for "Mistakes", "StyleRules", and "Skills" in Rust.
3.  **Database Integration:**
    -   Set up **SurrealDB** for storing structured metadata and relationships.
    -   Set up **Qdrant** for vector embeddings and semantic search.
4.  **Core Logic:** Implement the logic to ingest data into both databases and retrieve it based on query similarity.
5.  **Error Handling:** Ensure all outputs, especially errors, strictly follow the JSON format defined in the guidelines.

## User Stories
-   **As a user**, I want to run `lazydev add --category "rust" --type "mistake" --content "..."` so I can expand the knowledge base.
-   **As a user**, I want to run `lazydev context "how to handle errors in rust"` and get a JSON response with relevant mistakes and style rules.
-   **As a system**, I need to persist data across sessions using the configured databases.

## Technical Requirements
-   **Language:** Rust (Edition 2021+)
-   **Crates:**
    -   `clap` (v4+) for CLI parsing.
    -   `surrealdb` for graph/document storage.
    -   `qdrant-client` for vector search.
    -   `serde`, `serde_json` for serialization.
    -   `tokio` for async runtime.
    -   `anyhow` / `thiserror` for error handling.
    -   *Optional:* A local embedding model crate (e.g., `fastembed`) or an API client (e.g., OpenAI) for generating vectors. *For this MVP, we will assume a simple local embedding or a placeholder to focus on the architecture.*

## Constraints
-   Must compile to a single binary (or a binary + minimal config).
-   Databases should run locally (e.g., via Docker or embedded if possible, but Docker is standard for Qdrant/SurrealDB dev).
