use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

/// Validates user story creation parameters.
pub(crate) fn validate_user_story_params(title: &str, description: &str) -> anyhow::Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow::anyhow!("User story title cannot be empty"));
    }
    if description.trim().is_empty() {
        return Err(anyhow::anyhow!("User story description cannot be empty"));
    }
    if title.len() > 255 {
        return Err(anyhow::anyhow!("User story title too long (max 255 chars)"));
    }
    Ok(())
}

impl DB {
    /// Internal helper: creates a user story record without any relationships.
    pub(crate) async fn create_user_story_record(
        &self,
        user_story: &crate::models::UserStory,
    ) -> anyhow::Result<crate::models::UserStory> {
        let json = serde_json::to_value(user_story)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("user_story").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create user story"))?;
        let result: crate::models::UserStory = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a user story and RELATEs it to its parent project with bidirectional edges.
    pub async fn create_user_story(
        &self,
        title: &str,
        description: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::UserStory> {
        validate_user_story_params(title, description)?;

        let user_story = crate::models::UserStory {
            id: None,
            title: title.to_string(),
            description: description.to_string(),
        };
        let result = self.create_user_story_record(&user_story).await?;

        if let Some(usid) = result.id.as_ref() {
            self.link(project_id, "has_user_story", usid).await?;
            self.link(usid, "belongs_to_project", project_id).await?;
        }

        Ok(result)
    }

    /// Fetches a user story by record id.
    pub async fn get_user_story(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::UserStory>> {
        self.get_record("user_story", id).await
    }

    /// Returns all user stories (unfiltered).
    pub async fn list_user_stories(&self) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.list_records("user_story").await
    }

    /// Lists user stories under a project via graph traversal.
    pub async fn list_user_stories_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.query_graph_list(
            "SELECT ->has_user_story->user_story.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Links an existing task to a user story.
    pub async fn link_task_to_user_story(
        &self,
        task_id: &str,
        user_story_id: &str,
    ) -> anyhow::Result<()> {
        self.link(user_story_id, "has_task", task_id).await?;
        self.link(task_id, "belongs_to_story", user_story_id)
            .await?;
        Ok(())
    }

    /// Lists tasks linked to a user story.
    pub async fn list_tasks_by_user_story(
        &self,
        user_story_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Task>> {
        self.query_graph_list(
            "SELECT ->has_task->task.* AS items FROM ONLY type::record($usid)",
            "usid",
            user_story_id.to_string(),
            "items",
        )
        .await
    }

    /// Links an existing module to a user story.
    pub async fn link_module_to_user_story(
        &self,
        module_id: &str,
        user_story_id: &str,
    ) -> anyhow::Result<()> {
        self.link(user_story_id, "has_module", module_id).await?;
        self.link(module_id, "belongs_to_user_story", user_story_id)
            .await?;
        Ok(())
    }

    /// Links an existing submodule to a user story.
    pub async fn link_submodule_to_user_story(
        &self,
        submodule_id: &str,
        user_story_id: &str,
    ) -> anyhow::Result<()> {
        self.link(user_story_id, "has_submodule", submodule_id)
            .await?;
        self.link(submodule_id, "belongs_to_user_story", user_story_id)
            .await?;
        Ok(())
    }

    /// Lists modules linked to a user story.
    pub async fn list_modules_by_user_story(
        &self,
        user_story_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Module>> {
        self.query_graph_list(
            "SELECT ->has_module->module.* AS items FROM ONLY type::record($usid)",
            "usid",
            user_story_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists submodules linked to a user story.
    pub async fn list_submodules_by_user_story(
        &self,
        user_story_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.query_graph_list(
            "SELECT ->has_submodule->submodule.* AS items FROM ONLY type::record($usid)",
            "usid",
            user_story_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists user stories linked to a module.
    pub async fn list_user_stories_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.query_graph_list(
            "SELECT ->belongs_to_user_story->user_story.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists user stories linked to a submodule.
    pub async fn list_user_stories_by_submodule(
        &self,
        submodule_id: &str,
    ) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.query_graph_list(
            "SELECT ->belongs_to_user_story->user_story.* AS items FROM ONLY type::record($sid)",
            "sid",
            submodule_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists user stories linked to a task.
    pub async fn list_user_stories_by_task(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<crate::models::UserStory>> {
        self.query_graph_list(
            "SELECT ->belongs_to_story->user_story.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }

    /// Updates a user story's title or description.
    pub async fn update_user_story(
        &self,
        user_story_id: &str,
        title: Option<String>,
        description: Option<String>,
    ) -> anyhow::Result<Option<crate::models::UserStory>> {
        let key = user_story_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(user_story_id);

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
            return self.get_user_story(user_story_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("user_story", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes a user story by id.
    pub async fn delete_user_story(&self, user_story_id: &str) -> anyhow::Result<bool> {
        let key = user_story_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(user_story_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("user_story", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_user_story_params_accepts_valid_input() {
        validate_user_story_params("Valid Title", "Valid Description")
            .expect("should accept valid params");
    }

    #[test]
    fn validate_user_story_params_rejects_empty_title() {
        let err =
            validate_user_story_params("", "Description").expect_err("empty title should fail");
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn validate_user_story_params_rejects_whitespace_only_title() {
        let err = validate_user_story_params("   ", "Description")
            .expect_err("whitespace title should fail");
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn validate_user_story_params_rejects_empty_description() {
        let err =
            validate_user_story_params("Title", "").expect_err("empty description should fail");
        assert!(err.to_string().contains("description"));
    }

    #[test]
    fn validate_user_story_params_rejects_long_title() {
        let long_title = "a".repeat(256);
        let err = validate_user_story_params(&long_title, "Description")
            .expect_err("long title should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn validate_user_story_params_accepts_max_length_title() {
        let max_title = "a".repeat(255);
        validate_user_story_params(&max_title, "Description")
            .expect("255 char title should be accepted");
    }

    #[tokio::test]
    async fn test_delete_user_story_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let user_story = db
            .create_user_story("DeleteStory", "test", "project:1")
            .await
            .expect("create");
        let id = user_story.id.unwrap();

        let deleted = db.delete_user_story(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_user_story(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
