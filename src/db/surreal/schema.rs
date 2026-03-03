//! SurrealDB relation table schemas and table list for maintenance.

use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Table names used by purge_database. Keep in sync with DEFINE TABLE usage.
pub(crate) const TABLES: &[&str] = &[
    "project",
    "module",
    "submodule",
    "file",
    "task",
    "subtask",
    "todo_item",
    "context",
    "user_story",
    "epic",
];

/// Defines relation table schemas so Surrealist can visualize graph edges.
pub(crate) async fn define_schema(client: &Surreal<Any>) -> anyhow::Result<()> {
    client
        .query(
            "\
            DEFINE TABLE IF NOT EXISTS contains TYPE RELATION \
                IN project|module|submodule \
                OUT module|submodule|file;
            DEFINE TABLE IF NOT EXISTS has_task TYPE RELATION \
                IN project|user_story|epic OUT task;
            DEFINE TABLE IF NOT EXISTS belongs_to_project TYPE RELATION \
                IN task|context|user_story|epic OUT project;
            DEFINE TABLE IF NOT EXISTS belongs_to_module TYPE RELATION \
                IN task|context OUT module;
            DEFINE TABLE IF NOT EXISTS has_subtask TYPE RELATION \
                IN task OUT subtask;
            DEFINE TABLE IF NOT EXISTS belongs_to_task TYPE RELATION \
                IN subtask|context OUT task;
            DEFINE TABLE IF NOT EXISTS has_context TYPE RELATION \
                IN project|task|module|submodule|subtask|epic OUT context;
            DEFINE TABLE IF NOT EXISTS has_todo TYPE RELATION \
                IN project OUT todo_item;
            DEFINE TABLE IF NOT EXISTS has_user_story TYPE RELATION \
                IN project|epic OUT user_story;
            DEFINE TABLE IF NOT EXISTS belongs_to_story TYPE RELATION \
                IN task OUT user_story;
            DEFINE TABLE IF NOT EXISTS has_module TYPE RELATION \
                IN user_story OUT module;
            DEFINE TABLE IF NOT EXISTS has_submodule TYPE RELATION \
                IN user_story OUT submodule;
            DEFINE TABLE IF NOT EXISTS belongs_to_user_story TYPE RELATION \
                IN module|submodule OUT user_story;
            DEFINE TABLE IF NOT EXISTS has_epic TYPE RELATION \
                IN project OUT epic;
            DEFINE TABLE IF NOT EXISTS belongs_to_epic TYPE RELATION \
                IN user_story|task OUT epic;
            ",
        )
        .await?;
    Ok(())
}
