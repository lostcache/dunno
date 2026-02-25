use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::DB;

impl DB {
    /// Creates a module and RELATEs it to its parent project.
    pub async fn create_module(
        &self,
        name: &str,
        description: &str,
        project_id: &str,
    ) -> anyhow::Result<crate::models::Module> {
        let module = crate::models::Module {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = serde_json::to_value(&module)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("module").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create module"))?;
        let result: crate::models::Module = serde_json::from_value(surreal_to_json(val))?;
        let module_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Module missing id after create"))?;

        self.relate(project_id, "contains", module_id).await?;
        Ok(result)
    }

    /// Fetches a module by record id.
    pub async fn get_module(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Module>> {
        self.get_record("module", id).await
    }

    /// Lists modules under a project via graph traversal.
    pub async fn list_modules_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Module>> {
        self.query_graph_list(
            "SELECT ->contains->module.* AS items FROM ONLY type::record($pid)",
            "pid",
            project_id.to_string(),
            "items",
        )
        .await
    }

    /// Returns all modules (unfiltered).
    pub async fn list_modules(&self) -> anyhow::Result<Vec<crate::models::Module>> {
        self.list_records("module").await
    }

    /// Creates a submodule and RELATEs it to its parent module.
    pub async fn create_submodule(
        &self,
        name: &str,
        description: &str,
        module_id: &str,
    ) -> anyhow::Result<crate::models::Submodule> {
        let submodule = crate::models::Submodule {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            files: None,
        };
        let json = serde_json::to_value(&submodule)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("submodule").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create submodule"))?;
        let result: crate::models::Submodule = serde_json::from_value(surreal_to_json(val))?;
        let sub_id = result
            .id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Submodule missing id after create"))?;

        self.relate(module_id, "contains", sub_id).await?;
        Ok(result)
    }

    /// Fetches a submodule by record id.
    pub async fn get_submodule(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::Submodule>> {
        self.get_record("submodule", id).await
    }

    /// Returns all submodules.
    pub async fn list_submodules(&self) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.list_records("submodule").await
    }

    /// Lists submodules under a module via graph traversal.
    pub async fn list_submodules_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::Submodule>> {
        self.query_graph_list(
            "SELECT ->contains->submodule.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }
}
