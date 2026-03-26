use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

impl DB {
    /// Creates an issue record and optionally links it to a task.
    pub async fn create_issue(
        &self,
        description: &str,
        task_id: Option<&str>,
        plan: Option<&str>,
    ) -> anyhow::Result<crate::models::Issue> {
        let issue = crate::models::Issue {
            id: None,
            description: description.to_string(),
            status: crate::models::IssueStatus::Pending,
            plan: plan.map(|s| s.to_string()),
        };
        let json = serde_json::to_value(&issue)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("issue").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create issue"))?;
        let result: crate::models::Issue = serde_json::from_value(surreal_to_json(val))?;

        if let (Some(tid), Some(iid)) = (task_id, result.id.as_ref()) {
            self.link(tid, "has_issue", iid).await?;
            self.link(iid, "belongs_to_task", tid).await?;
        }

        Ok(result)
    }

    /// Fetches an issue by record id.
    pub async fn get_issue(&self, id: &str) -> anyhow::Result<Option<crate::models::Issue>> {
        self.get_record("issue", id).await
    }

    /// Returns all issues (unfiltered).
    pub async fn list_issues(&self) -> anyhow::Result<Vec<crate::models::Issue>> {
        self.list_records("issue").await
    }

    /// Lists issues linked to a task via has_issue.
    pub async fn list_issues_by_task(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Issue>> {
        self.query_graph_list(
            "SELECT ->has_issue->issue.* AS items FROM ONLY type::record($tid)",
            "tid",
            task_id.to_string(),
            "items",
        )
        .await
    }

    /// Updates an issue's title, description, status, or plan.
    pub async fn update_issue(
        &self,
        issue_id: &str,
        description: Option<String>,
        status: Option<crate::models::IssueStatus>,
        plan: Option<String>,
    ) -> anyhow::Result<Option<crate::models::Issue>> {
        let key = issue_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(issue_id);

        let mut patch = serde_json::Map::new();
        if let Some(description) = description {
            patch.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }
        if let Some(status) = status {
            patch.insert("status".to_string(), serde_json::to_value(status)?);
        }
        if let Some(plan) = plan {
            patch.insert("plan".to_string(), serde_json::Value::String(plan));
        }

        if patch.is_empty() {
            return self.get_issue(issue_id).await;
        }

        let updated: Option<surrealdb::types::Value> = self
            .client
            .update(("issue", key))
            .merge(json_to_surreal(serde_json::Value::Object(patch)))
            .await?;

        if let Some(val) = updated {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    /// Deletes an issue by record id.
    pub async fn delete_issue(&self, issue_id: &str) -> anyhow::Result<bool> {
        let key = issue_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(issue_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("issue", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_issue_standalone() {
        let db = DB::new("mem://").await.expect("init db");
        let issue = db
            .create_issue("Users cannot log in", None, None)
            .await
            .expect("create issue");
        assert_eq!(issue.description, "Users cannot log in");
        assert_eq!(issue.status, crate::models::IssueStatus::Pending);
        assert!(issue.id.is_some());
    }

    #[tokio::test]
    async fn test_create_issue_linked_to_task() {
        let db = DB::new("mem://").await.expect("init db");

        let project = db
            .create_project(&crate::models::Project {
                id: None,
                name: "Test Project".to_string(),
                description: "desc".to_string(),
            })
            .await
            .expect("create project");
        let project_id = project.id.unwrap();

        let task = db
            .create_task("Fix auth", "Auth broken", None, Some(&project_id))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        let issue = db
            .create_issue("Tokens expire too early", Some(&task_id), None)
            .await
            .expect("create issue");

        let issues = db
            .list_issues_by_task(&task_id)
            .await
            .expect("list issues");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, issue.id);
    }

    #[tokio::test]
    async fn test_delete_issue() {
        let db = DB::new("mem://").await.expect("init db");
        let issue = db
            .create_issue("desc", None, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let deleted = db.delete_issue(&id).await.expect("delete issue");
        assert!(deleted);

        let after = db.get_issue(&id).await.expect("get issue");
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_issue() {
        let db = DB::new("mem://").await.expect("init db");
        // ensure table exists
        db.create_issue("seed", None, None)
            .await
            .expect("seed issue");

        let deleted = db
            .delete_issue("issue:nonexistent12345")
            .await
            .expect("no error on missing");
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        let db = DB::new("mem://").await.expect("init db");
        let issue = db
            .create_issue("desc", None, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let updated = db
            .update_issue(&id, None, Some(crate::models::IssueStatus::Active), None)
            .await
            .expect("update issue")
            .unwrap();
        assert_eq!(updated.status, crate::models::IssueStatus::Active);
    }

    #[tokio::test]
    async fn test_update_issue_plan() {
        let db = DB::new("mem://").await.expect("init db");
        let issue = db
            .create_issue("desc", None, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let updated = db
            .update_issue(&id, None, None, Some("Fix the auth module".to_string()))
            .await
            .expect("update issue")
            .unwrap();
        assert_eq!(updated.plan.as_deref(), Some("Fix the auth module"));
    }
}
