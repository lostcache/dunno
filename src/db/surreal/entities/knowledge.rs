use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::DB;

impl DB {
    /// Creates a new mistake record.
    pub async fn create_mistake(
        &self,
        mistake: &crate::models::Mistake,
    ) -> anyhow::Result<crate::models::Mistake> {
        let json = serde_json::to_value(mistake)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("mistake").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create mistake"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a mistake by record id.
    pub async fn get_mistake(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Mistake>> {
        self.get_record("mistake", id).await
    }

    /// Returns all mistakes.
    pub async fn list_mistakes(&self) -> anyhow::Result<Vec<crate::models::Mistake>> {
        self.list_records("mistake").await
    }

    /// Creates a new style rule record.
    pub async fn create_style_rule(
        &self,
        rule: &crate::models::StyleRule,
    ) -> anyhow::Result<crate::models::StyleRule> {
        let json = serde_json::to_value(rule)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("style_rule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create style rule"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a style rule by record id.
    pub async fn get_style_rule(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::StyleRule>> {
        self.get_record("style_rule", id).await
    }

    /// Returns all style rules.
    pub async fn list_style_rules(&self) -> anyhow::Result<Vec<crate::models::StyleRule>> {
        self.list_records("style_rule").await
    }

    /// Creates a new security detail record.
    pub async fn create_security_detail(
        &self,
        detail: &crate::models::SecurityDetail,
    ) -> anyhow::Result<crate::models::SecurityDetail> {
        let json = serde_json::to_value(detail)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("security_detail").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create security detail"))?;
        Ok(serde_json::from_value(surreal_to_json(val))?)
    }

    /// Fetches a security detail by record id.
    pub async fn get_security_detail(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::SecurityDetail>> {
        self.get_record("security_detail", id).await
    }

    /// Returns all security details.
    pub async fn list_security_details(
        &self,
    ) -> anyhow::Result<Vec<crate::models::SecurityDetail>> {
        self.list_records("security_detail").await
    }

    /// Creates a knowledge edge from a structural node to a knowledge node.
    /// Also creates reverse belongs_to edges from the knowledge node.
    pub async fn link_context(&self, from_id: &str, to_id: &str) -> anyhow::Result<()> {
        let edge = if to_id.starts_with("mistake:") {
            "has_mistake"
        } else if to_id.starts_with("style_rule:") {
            "has_style"
        } else if to_id.starts_with("security_detail:") {
            "has_security_detail"
        } else {
            return Err(anyhow::anyhow!(
                "link_context: to_id must be mistake:, style_rule:, or security_detail: record; got {:?}",
                to_id
            ));
        };
        self.relate(from_id, edge, to_id).await?;

        let hierarchy = self.resolve_structural_hierarchy(from_id).await?;
        if let Some(id) = hierarchy.project_id {
            self.relate(to_id, "belongs_to_project", &id).await?;
        }
        if let Some(id) = hierarchy.module_id {
            self.relate(to_id, "belongs_to_module", &id).await?;
        }
        if let Some(id) = hierarchy.task_id {
            self.relate(to_id, "belongs_to_task", &id).await?;
        }
        Ok(())
    }

    /// Returns all structural node ids that the given knowledge record points to.
    pub async fn get_belongs_to_targets(
        &self,
        knowledge_record_id: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        for (edge, table) in [
            ("belongs_to_project", "project"),
            ("belongs_to_module", "module"),
            ("belongs_to_task", "task"),
        ] {
            let id = self
                .first_record_id_from_query(
                    &format!("SELECT ->{edge}->{table}.* AS out FROM ONLY type::record($kid)"),
                    "kid",
                    knowledge_record_id.to_string(),
                    "out",
                )
                .await?;
            if let Some(id) = id {
                out.push(id);
            }
        }
        Ok(out)
    }
}
