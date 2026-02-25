//! SurrealDB relation table schemas and table list for maintenance.

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

/// Table names used by purge_database. Keep in sync with DEFINE TABLE usage.
pub(crate) const TABLES: &[&str] = &[
    "project",
    "module",
    "submodule",
    "file",
    "task",
    "subtask",
    "todo_item",
    "mistake",
    "style_rule",
    "security_detail",
];

/// Defines relation table schemas so Surrealist can visualize graph edges.
pub(crate) async fn define_schema(client: &Surreal<Any>) -> anyhow::Result<()> {
    client
        .query(
            "\
            DEFINE TABLE IF NOT EXISTS contains TYPE RELATION \
                IN project|module|submodule \
                OUT module|submodule|file;\
            DEFINE TABLE IF NOT EXISTS has_task TYPE RELATION \
                IN project OUT task;\
            DEFINE TABLE IF NOT EXISTS belongs_to_project TYPE RELATION \
                IN task|mistake|style_rule|security_detail OUT project;\
            DEFINE TABLE IF NOT EXISTS belongs_to_module TYPE RELATION \
                IN task|mistake|style_rule|security_detail OUT module;\
            DEFINE TABLE IF NOT EXISTS has_subtask TYPE RELATION \
                IN task OUT subtask;\
            DEFINE TABLE IF NOT EXISTS belongs_to_task TYPE RELATION \
                IN subtask|mistake|style_rule|security_detail OUT task;\
            DEFINE TABLE IF NOT EXISTS has_mistake TYPE RELATION \
                IN project|task|module|submodule|subtask OUT mistake;\
            DEFINE TABLE IF NOT EXISTS has_style TYPE RELATION \
                IN project|task|module|submodule|subtask OUT style_rule;\
            DEFINE TABLE IF NOT EXISTS has_security_detail TYPE RELATION \
                IN project|task|module|submodule|subtask OUT security_detail;\
            DEFINE TABLE IF NOT EXISTS has_todo TYPE RELATION \
                IN project OUT todo_item;\
            ",
        )
        .await?;
    Ok(())
}
