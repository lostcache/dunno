//! SurrealDB relation table schemas and table list for maintenance.

use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Table names used by purge_database. Keep in sync with DEFINE TABLE usage.
pub(crate) const TABLES: &[&str] = &[
    "project",
    "module",
    "file",
    "task",
    "todo_item",
    "context",
    "user_story",
    "epic",
    "persona",
    "workflow",
    "issue",
];

/// Defines relation table schemas so Surrealist can visualize graph edges.
pub(crate) async fn define_schema(client: &Surreal<Any>) -> anyhow::Result<()> {
    client
        .query(
            "\
            DEFINE TABLE IF NOT EXISTS contains TYPE RELATION \
                IN project|module \
                OUT module|file;
            DEFINE TABLE IF NOT EXISTS has_task TYPE RELATION \
                IN project|user_story|epic OUT task;
            DEFINE TABLE OVERWRITE belongs_to_project TYPE RELATION \
                IN task|context|user_story|epic|file|module|persona|workflow|issue OUT project;
            DEFINE TABLE IF NOT EXISTS belongs_to_module TYPE RELATION \
                IN task|context|file|module OUT module;
            DEFINE TABLE IF NOT EXISTS has_context TYPE RELATION \
                IN project|task|module|epic|file OUT context;
            DEFINE TABLE IF NOT EXISTS has_todo TYPE RELATION \
                IN project OUT todo_item;
            DEFINE TABLE IF NOT EXISTS has_user_story TYPE RELATION \
                IN project|epic OUT user_story;
            DEFINE TABLE IF NOT EXISTS belongs_to_story TYPE RELATION \
                IN task OUT user_story;
            DEFINE TABLE IF NOT EXISTS belongs_to_user_story TYPE RELATION \
                IN module OUT user_story;
            DEFINE TABLE IF NOT EXISTS has_epic TYPE RELATION \
                IN project OUT epic;
            DEFINE TABLE IF NOT EXISTS belongs_to_epic TYPE RELATION \
                IN user_story|task OUT epic;
            DEFINE TABLE IF NOT EXISTS has_persona TYPE RELATION \
                IN project OUT persona;
            DEFINE TABLE IF NOT EXISTS has_workflow TYPE RELATION \
                IN project OUT workflow;
            DEFINE TABLE IF NOT EXISTS has_issue TYPE RELATION \
                IN task OUT issue;
            DEFINE TABLE IF NOT EXISTS belongs_to_task TYPE RELATION \
                IN file|context|issue OUT task;
            DEFINE INDEX IF NOT EXISTS project_name_idx ON project COLUMNS name UNIQUE;
            ",
        )
        .await?;
    Ok(())
}
