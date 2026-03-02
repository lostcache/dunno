use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::DB;

impl DB {
    /// Internal helper: creates a todo item record without any relationships.
    pub(crate) async fn create_todo_record(
        &self,
        todo: &crate::models::TodoItem,
    ) -> anyhow::Result<crate::models::TodoItem> {
        let json = serde_json::to_value(todo)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("todo_item").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create todo item"))?;
        let result: crate::models::TodoItem = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a todo item and optionally RELATEs it to a project via `has_todo`.
    pub async fn create_todo(
        &self,
        content: &str,
        project_id: Option<&str>,
    ) -> anyhow::Result<crate::models::TodoItem> {
        let todo = crate::models::TodoItem {
            id: None,
            content: content.to_string(),
        };
        let result = self.create_todo_record(&todo).await?;
        if let (Some(pid), Some(tid)) = (project_id, result.id.as_ref()) {
            self.link(pid, "has_todo", tid).await?;
        }
        Ok(result)
    }

    /// Fetches a todo item by record id.
    pub async fn get_todo(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::TodoItem>> {
        self.get_record("todo_item", id).await
    }

    /// Lists todo items for a project via graph traversal.
    pub async fn list_todos_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::TodoItem>> {
        self.query_graph_list(
            "SELECT ->has_todo->todo_item.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all todo items (unfiltered).
    pub async fn list_todos(&self) -> anyhow::Result<Vec<crate::models::TodoItem>> {
        self.list_records("todo_item").await
    }
}
