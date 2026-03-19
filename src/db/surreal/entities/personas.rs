use crate::db::surreal::DB;
use crate::db::surreal::util::{json_to_surreal, surreal_to_json};

/// Validates persona creation parameters.
pub(crate) fn validate_persona_params(name: &str, content: &str) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!("Persona name cannot be empty"));
    }
    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("Persona content cannot be empty"));
    }
    if name.len() > 255 {
        return Err(anyhow::anyhow!("Persona name too long (max 255 chars)"));
    }
    Ok(())
}

impl DB {
    /// Internal helper: creates a persona record without any relationships.
    pub(crate) async fn create_persona_record(
        &self,
        persona: &crate::models::Persona,
    ) -> anyhow::Result<crate::models::Persona> {
        let json = serde_json::to_value(persona)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("persona").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create persona"))?;
        let result: crate::models::Persona = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a persona and RELATEs it to its parent project with bidirectional edges.
    pub async fn create_persona(
        &self,
        name: &str,
        content: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Persona> {
        validate_persona_params(name, content)?;

        let persona = crate::models::Persona {
            id: None,
            name: name.to_string(),
            content: content.to_string(),
        };
        let result = self.create_persona_record(&persona).await?;

        if let Some(pid) = result.id.as_ref() {
            self.link(project_id, "has_persona", pid).await?;
            self.link(pid, "belongs_to_project", project_id).await?;
        }

        Ok(result)
    }

    /// Fetches a persona by record id.
    pub async fn get_persona(&self, id: &str) -> anyhow::Result<Option<crate::models::Persona>> {
        self.get_record("persona", id).await
    }

    /// Returns all personas (unfiltered).
    pub async fn list_personas(&self) -> anyhow::Result<Vec<crate::models::Persona>> {
        self.list_records("persona").await
    }

    /// Lists personas under a project via graph traversal.
    pub async fn list_personas_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Persona>> {
        self.query_graph_list(
            "SELECT ->has_persona->persona.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Deletes a persona by id.
    pub async fn delete_persona(&self, persona_id: &str) -> anyhow::Result<bool> {
        let key = persona_id
            .split_once(':')
            .map(|(_, key)| key)
            .unwrap_or(persona_id);

        let deleted: Option<surrealdb::types::Value> =
            self.client.delete(("persona", key)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_persona_params_accepts_valid_input() {
        validate_persona_params("Valid Name", "Valid content")
            .expect("should accept valid params");
    }

    #[test]
    fn validate_persona_params_rejects_empty_name() {
        let err = validate_persona_params("", "content").expect_err("empty name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_persona_params_rejects_whitespace_only_name() {
        let err =
            validate_persona_params("   ", "content").expect_err("whitespace name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validate_persona_params_rejects_empty_content() {
        let err = validate_persona_params("Name", "").expect_err("empty content should fail");
        assert!(err.to_string().contains("content"));
    }

    #[test]
    fn validate_persona_params_rejects_long_name() {
        let long_name = "a".repeat(256);
        let err =
            validate_persona_params(&long_name, "content").expect_err("long name should fail");
        assert!(err.to_string().contains("too long"));
    }

    #[tokio::test]
    async fn test_delete_persona_success() {
        let db = DB::new("mem://").await.expect("Failed to init DB");
        let persona = db
            .create_persona("DeletePersona", "test content", "project:1")
            .await
            .expect("create");
        let id = persona.id.unwrap();

        let deleted = db.delete_persona(&id).await.expect("delete");
        assert!(deleted);

        let fetched = db.get_persona(&id).await.expect("fetch");
        assert!(fetched.is_none());
    }
}
