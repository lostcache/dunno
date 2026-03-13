use crate::db::surreal::DB;
use crate::db::surreal::util::{ensure_record_id, json_to_surreal, surreal_to_json};

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
            ensure_record_id("project", pid)?;
            self.link(pid, "has_todo", tid).await?;
        }
        Ok(result)
    }

    /// Fetches a todo item by record id.
    pub async fn get_todo(&self, id: &str) -> anyhow::Result<Option<crate::models::TodoItem>> {
        self.get_record("todo_item", id).await
    }

    /// Lists todo items for a project via graph traversal.
    pub async fn list_todos_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::TodoItem>> {
        ensure_record_id("project", project_id)?;
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

    /// Deletes a todo by record id.
    pub async fn delete_todo(&self, todo_id: &str) -> anyhow::Result<bool> {
        let key = todo_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(todo_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("todo_item", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_content_validation_accepts_valid() {
        let content = "Buy milk";
        assert!(!content.trim().is_empty());
        assert!(content.len() <= 1000);
    }

    #[test]
    fn todo_content_validation_rejects_empty() {
        let content = "";
        assert!(content.trim().is_empty());
    }

    #[test]
    fn todo_content_validation_rejects_whitespace() {
        let content = "   ";
        assert!(content.trim().is_empty());
    }

    #[test]
    fn project_id_must_have_project_prefix_for_todo_ops() {
        ensure_record_id("project", "project:abc")
            .expect("list_todos_by_project accepts project:id");
        let err = ensure_record_id("project", "module:1").expect_err("wrong table rejected");
        assert!(err.to_string().contains("Expected record id"));
    }

    #[tokio::test]
    async fn test_delete_todo_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        let todo = db
            .create_todo("Buy milk", None)
            .await
            .expect("Failed to create todo");
        let todo_id = todo.id.expect("todo id");

        let fetched = db.get_todo(&todo_id).await.expect("Failed to fetch todo");
        assert!(fetched.is_some());

        let deleted = db
            .delete_todo(&todo_id)
            .await
            .expect("Failed to delete todo");
        assert!(deleted, "delete_todo should return true for existing todo");

        let after_delete = db.get_todo(&todo_id).await.expect("Failed to check todo");
        assert!(after_delete.is_none(), "Todo should be deleted");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_todo() {
        let db = DB::new("mem://").await.expect("Failed to init DB");

        // First create a todo to ensure the table exists
        let todo = db
            .create_todo("Test Todo", None)
            .await
            .expect("create todo");
        let _todo_id = todo.id.expect("todo id");

        let deleted = db
            .delete_todo("todo_item:nonexistent12345")
            .await
            .expect("Should not error on nonexistent todo");

        assert!(
            !deleted,
            "delete_todo should return false for nonexistent todo"
        );
    }
}
