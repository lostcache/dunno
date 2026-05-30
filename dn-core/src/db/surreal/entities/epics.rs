use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

/// Validates epic creation parameters.
pub(crate) fn validate_epic_params(title: &str, description: &str) -> anyhow::Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow::anyhow!("Epic title cannot be empty"));
    }
    if description.trim().is_empty() {
        return Err(anyhow::anyhow!("Epic description cannot be empty"));
    }
    if title.len() > 255 {
        return Err(anyhow::anyhow!("Epic title too long (max 255 chars)"));
    }
    Ok(())
}

impl DB {
    /// Internal helper: creates an epic record without any relationships.
    pub(crate) async fn create_epic_record(
        &self,
        epic: &crate::models::Epic,
    ) -> anyhow::Result<crate::models::Epic> {
        let json = serde_json::to_value(epic)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("epic").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create epic"))?;
        let result: crate::models::Epic = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates an epic and RELATEs it to its parent project with bidirectional edges.
    pub async fn create_epic(
        &self,
        title: &str,
        description: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Epic> {
        validate_epic_params(title, description)?;

        let epic = crate::models::Epic {
            id: None,
            title: title.to_string(),
            description: description.to_string(),
        };
        let result = self.create_epic_record(&epic).await?;

        if let Some(eid) = result.id.as_ref() {
            self.link(project_id, "has_epic", eid).await?;
            self.link(eid, "belongs_to_project", project_id).await?;
        }

        Ok(result)
    }

    /// Fetches an epic by record id.
    pub async fn get_epic(&self, id: &str) -> anyhow::Result<Option<crate::models::Epic>> {
        self.get_record("epic", id).await
    }

    /// Returns all epics (unfiltered).
    pub async fn list_epics(&self) -> anyhow::Result<Vec<crate::models::Epic>> {
        self.list_records("epic").await
    }

    /// Lists epics under a project via graph traversal.
    pub async fn list_epics_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Epic>> {
        self.query_graph_list(
            "SELECT ->has_epic->epic.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Links an existing user story to an epic.
    pub async fn link_user_story_to_epic(
        &self,
        user_story_id: &str,
        epic_id: &str,
    ) -> anyhow::Result<()> {
        self.link(epic_id, "has_user_story", user_story_id).await?;
        self.link(user_story_id, "belongs_to_epic", epic_id).await?;
        Ok(())
    }

    /// Links an existing task to an epic.
    pub async fn link_task_to_epic(&self, task_id: &str, epic_id: &str) -> anyhow::Result<()> {
        self.link(epic_id, "has_task", task_id).await?;
        self.link(task_id, "belongs_to_epic", epic_id).await?;
        Ok(())
    }

    /// Lists user stories linked to an epic.
    pub async fn list_user_stories_by_epic(
        &self,
        epic_id: &str,
    ) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.query_graph_list(
            "SELECT ->has_user_story->user_story.* AS items FROM ONLY type::record($eid)",
            "eid",
            epic_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists tasks linked to an epic.
    pub async fn list_tasks_by_epic(
        &self,
        epic_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Task>> {
        self.query_graph_list(
            "SELECT ->has_task->task.* AS items FROM ONLY type::record($eid)",
            "eid",
            epic_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists epics linked to a user story.
    pub async fn list_epics_by_user_story(
        &self,
        user_story_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Epic>> {
        self.query_graph_list(
            "SELECT ->belongs_to_epic->epic.* AS items FROM ONLY type::record($usid)",
            "usid",
            user_story_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists epics linked to a task.
    pub async fn list_epics_by_task(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Epic>> {
        self.query_graph_list(
            "SELECT ->belongs_to_epic->epic.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }

    /// Gets context only for the specific epic node.
    pub async fn get_epic_context_node(
        &self,
        epic_id: &str,
    ) -> anyhow::Result<crate::models::EpicContext> {
        let epic = self
            .get_epic(epic_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Epic not found: {}", epic_id))?;
        let contexts = self.get_linked_context(epic_id).await?;
        Ok(crate::models::EpicContext {
            persona: vec![],
            workflow: vec![],
            epic,
            contexts,
        })
    }

    /// Gets full inherited context for an epic (Project -> Epic).
    pub async fn get_epic_context_full(
        &self,
        epic_id: &str,
    ) -> anyhow::Result<crate::models::EpicContext> {
        let mut ctx = self.get_epic_context_node(epic_id).await?;

        // Resolve ancestry for this epic
        let ancestry = self.resolve_structural_ancestry(epic_id).await?;

        for pid in &ancestry.project_ids {
            ctx.contexts.extend(self.get_linked_context(pid).await?);
        }

        // Deduplicate contexts by ID
        let mut seen = std::collections::HashSet::new();
        ctx.contexts.retain(|c| {
            if let Some(id) = &c.id {
                seen.insert(id.clone())
            } else {
                true
            }
        });

        for pid in &ancestry.project_ids {
            ctx.persona
                .extend(self.list_personas_by_project(pid).await?);
            ctx.workflow
                .extend(self.list_workflows_by_project(pid).await?);
        }

        Ok(ctx)
    }

    /// Gets context for an epic, optionally including parent hierarchy.
    pub async fn get_epic_context(
        &self,
        epic_id: &str,
        full: bool,
    ) -> anyhow::Result<crate::models::EpicContext> {
        if full {
            self.get_epic_context_full(epic_id).await
        } else {
            self.get_epic_context_node(epic_id).await
        }
    }

    /// Updates an epic's title or description.
    pub async fn update_epic(
        &self,
        epic_id: &str,
        title: Option<String>,
        description: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Epic>> {
        let key = epic_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(epic_id);

        let mut patch = serde_json::Map::new();
        if let Some(title) = title {
            patch.insert("title".to_string(), serde_json::Value::String(title));
        }
        if let Some(description) = description {
            patch.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        if patch.is_empty() {
            return self.get_epic(epic_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("epic", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes an epic by id.
    pub async fn delete_epic(&self, epic_id: &str) -> anyhow::Result<bool> {
        let key = epic_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(epic_id);

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("epic", key)).await?;
        Ok(deleted.is_some())
    }
}

/// Returns full epic context including epic details and linked knowledge.
pub async fn get_epic_context_json(
    epic_id: &str,
    full: bool,
    db: &DB,
) -> anyhow::Result<crate::models::EpicContext> {
    db.get_epic_context(epic_id, full).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_epic_params_accepts_valid_input() {
        validate_epic_params("Valid Title", "Valid Description")
            .expect("should accept valid params");
    }

    #[test]
    fn validate_epic_params_rejects_empty_title() {
        let err = validate_epic_params("", "Description").expect_err("empty title should fail");
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn validate_epic_params_rejects_whitespace_only_title() {
        let err =
            validate_epic_params("   ", "Description").expect_err("whitespace title should fail");
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn validate_epic_params_rejects_empty_description() {
        let err = validate_epic_params("Title", "").expect_err("empty description should fail");
        assert!(err.to_string().contains("description"));
    }

    #[test]
    fn validate_epic_params_rejects_long_title() {
        let long_title = "a".repeat(256);
        let err =
            validate_epic_params(&long_title, "Description").expect_err("long title should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn validate_epic_params_accepts_max_length_title() {
        let max_title = "a".repeat(255);
        validate_epic_params(&max_title, "Description").expect("255 char title should be accepted");
    }

    #[tokio::test]
    async fn test_delete_epic_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let epic = db
            .create_epic("DeleteEpic", "test", "project:1")
            .await
            .expect("create");
        let id = epic.id.unwrap();

        let deleted = db.delete_epic(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_epic(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
