use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

impl DB {
    /// Creates an issue record, links it to a project, and optionally links it to a task.
    pub async fn create_issue(
        &self,
        description: &str,
        task_id: Option<&str>,
        plan: Option<&str>,
        project_id: &str,
        verification: Option<&str>,
    ) -> anyhow::Result<crate::models::Issue> {
        let issue = crate::models::Issue {
            id: None,
            description: description.to_string(),
            status: crate::models::IssueStatus::Pending,
            plan: plan.map(|s| s.to_string()),
            verification: verification.map(|s| s.to_string()),
        };
        let json = serde_json::to_value(&issue)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("issue").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create issue"))?;
        let result: crate::models::Issue = serde_json::from_value(surreal_to_json(val))?;

        let iid = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Created issue has no id"))?;

        self.link(iid, "belongs_to_project", project_id).await?;

        if let Some(tid) = task_id {
            self.link(tid, "has_issue", iid).await?;
            self.link(iid, "belongs_to_task", tid).await?;
        }

        Ok(result)
    }

    /// Lists issues linked to a project via belongs_to_project.
    pub async fn list_issues_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Issue>> {
        // Use a direct subquery instead of backward graph traversal to avoid
        // result-format mismatches (the backward traversal can return null/nested
        // tuples that query_graph_list mis-handles, as seen with the todo issue).
        let mut res = self
            .client
            .query("SELECT * FROM issue WHERE id INSIDE (SELECT VALUE in FROM belongs_to_project WHERE out = type::record($pid))")
            .bind(("pid", project_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
        let mut out = Vec::with_capacity(rows.len());
        for val in rows {
            let json = crate::db::surreal::util::surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
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
        verification: Option<String>,
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
        if let Some(verification) = verification {
            patch.insert(
                "verification".to_string(),
                serde_json::Value::String(verification),
            );
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

        let deleted: Option<surrealdb::types::Value> = self.client.delete(("issue", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_project(db: &DB, name: &str) -> String {
        db.create_project(&crate::models::Project {
            id: None,
            name: name.to_string(),
            description: "desc".to_string(),
        })
        .await
        .expect("create project")
        .id
        .unwrap()
    }

    #[tokio::test]
    async fn test_create_issue_standalone() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("Users cannot log in", None, None, &pid, None)
            .await
            .expect("create issue");
        assert_eq!(issue.description, "Users cannot log in");
        assert_eq!(issue.status, crate::models::IssueStatus::Pending);
        assert!(issue.id.is_some());
    }

    #[tokio::test]
    async fn test_create_issue_linked_to_task() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;

        let task = db
            .create_task("Fix auth", "Auth broken", None, Some(&pid))
            .await
            .expect("create task");
        let task_id = task.id.unwrap();

        let issue = db
            .create_issue("Tokens expire too early", Some(&task_id), None, &pid, None)
            .await
            .expect("create issue");

        let issues = db.list_issues_by_task(&task_id).await.expect("list issues");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, issue.id);
    }

    #[tokio::test]
    async fn test_create_issue_linked_to_project() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Proj Link Test").await;

        let issue = db
            .create_issue("Project-level issue", None, None, &pid, None)
            .await
            .expect("create issue");
        assert!(issue.id.is_some());

        let issues = db
            .list_issues_by_project(&pid)
            .await
            .expect("list issues by project");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, issue.id);
    }

    #[tokio::test]
    async fn test_list_issues_by_project() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Multi Issue Project").await;
        let other_pid = make_project(&db, "Other Project").await;

        db.create_issue("Issue A", None, None, &pid, None)
            .await
            .expect("create issue a");
        db.create_issue("Issue B", None, None, &pid, None)
            .await
            .expect("create issue b");
        db.create_issue("Issue C other project", None, None, &other_pid, None)
            .await
            .expect("create issue c");

        let issues = db
            .list_issues_by_project(&pid)
            .await
            .expect("list issues by project");
        assert_eq!(issues.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_issue() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("desc", None, None, &pid, None)
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
        let pid = make_project(&db, "Test Project").await;
        db.create_issue("seed", None, None, &pid, None)
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
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("desc", None, None, &pid, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let updated = db
            .update_issue(
                &id,
                None,
                Some(crate::models::IssueStatus::Active),
                None,
                None,
            )
            .await
            .expect("update issue")
            .unwrap();
        assert_eq!(updated.status, crate::models::IssueStatus::Active);
    }

    #[tokio::test]
    async fn test_update_issue_plan() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("desc", None, None, &pid, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let updated = db
            .update_issue(
                &id,
                None,
                None,
                Some("Fix the auth module".to_string()),
                None,
            )
            .await
            .expect("update issue")
            .unwrap();
        assert_eq!(updated.plan.as_deref(), Some("Fix the auth module"));
    }

    #[tokio::test]
    async fn test_create_issue_with_verification() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("desc", None, None, &pid, Some("Check login flow"))
            .await
            .expect("create issue");
        assert_eq!(issue.verification.as_deref(), Some("Check login flow"));
    }

    #[tokio::test]
    async fn test_update_issue_verification() {
        let db = DB::new("mem://").await.expect("init db");
        let pid = make_project(&db, "Test Project").await;
        let issue = db
            .create_issue("desc", None, None, &pid, None)
            .await
            .expect("create issue");
        let id = issue.id.unwrap();

        let updated = db
            .update_issue(&id, None, None, None, Some("Updated check".to_string()))
            .await
            .expect("update issue")
            .unwrap();
        assert_eq!(updated.verification.as_deref(), Some("Updated check"));
    }
}
