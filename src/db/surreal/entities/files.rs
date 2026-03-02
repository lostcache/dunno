use crate::db::surreal::convert::{json_to_surreal, surreal_to_json};
use crate::db::surreal::DB;

impl DB {
    /// Internal helper: creates a file record without any relationships.
    pub(crate) async fn create_file_record(
        &self,
        file: &crate::models::File,
    ) -> anyhow::Result<crate::models::File> {
        let json = serde_json::to_value(file)?;
        let value = json_to_surreal(json);
        let created: Option<surrealdb::types::Value> =
            self.client.create("file").content(value).await?;
        let val = created.ok_or_else(|| anyhow::anyhow!("Failed to create file"))?;
        let result: crate::models::File = serde_json::from_value(surreal_to_json(val))?;
        Ok(result)
    }

    /// Creates a file and optionally RELATEs it to a parent (module or submodule).
    pub async fn create_file(
        &self,
        name: &str,
        path: &str,
        parent_id: Option<&str>,
    ) -> anyhow::Result<crate::models::File> {
        let file = crate::models::File {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
        };
        let result = self.create_file_record(&file).await?;
        if let (Some(pid), Some(fid)) = (parent_id, result.id.as_ref()) {
            self.link(pid, "contains", fid).await?;
        }
        Ok(result)
    }

    /// Fetches a file by record id.
    pub async fn get_file(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<crate::models::File>> {
        self.get_record("file", id).await
    }

    /// Returns all files.
    pub async fn list_files(&self) -> anyhow::Result<Vec<crate::models::File>> {
        self.list_records("file").await
    }

    /// Lists files under a module via graph traversal.
    pub async fn list_files_by_module(
        &self,
        module_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($mid)",
            "mid",
            module_id.to_string(),
            "items",
        )
        .await
    }

    /// Lists files under a submodule via graph traversal.
    pub async fn list_files_by_submodule(
        &self,
        submodule_id: &str,
    ) -> anyhow::Result<Vec<crate::models::File>> {
        self.query_graph_list(
            "SELECT ->contains->file.* AS items FROM ONLY type::record($sid)",
            "sid",
            submodule_id.to_string(),
            "items",
        )
        .await
    }
}

/// Runs the file context SurrealQL and returns a flattened JSON list of knowledge nodes.
pub async fn get_file_context_json(
    file_id: &str,
    db: &crate::db::DB,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let sql = r#"
        LET $f = type::record($fid);

        LET $f_ctx = (SELECT ->has_context->context.* FROM ONLY $f);
        RETURN [$f_ctx];
    "#;

    let raw = db
        .query_raw_json(sql, "fid", file_id.to_string(), 2)
        .await?;
    Ok(crate::db::surreal::flatten_context::flatten_context_result(raw))
}

