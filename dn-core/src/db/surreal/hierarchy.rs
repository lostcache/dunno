use crate::db::surreal::DB;
use crate::db::surreal::util::surreal_to_json;

/// Full structural ancestry for a node (used when creating reverse knowledge edges).
#[derive(Debug, Default, Clone)]
pub(crate) struct StructuralAncestry {
    pub(crate) project_ids: Vec<String>,
    /// All ancestor module IDs (self + all parents up the chain).
    pub(crate) module_ids: Vec<String>,
    pub(crate) task_ids: Vec<String>,
    pub(crate) epic_ids: Vec<String>,
}

impl DB {
    /// Resolves the full structural ancestry for a structural node.
    pub(crate) async fn resolve_structural_ancestry(
        &self,
        from_id: &str,
    ) -> anyhow::Result<StructuralAncestry> {
        let table = from_id.split_once(':').map(|(t, _)| t).unwrap_or("");
        match table {
            "project" => Ok(StructuralAncestry {
                project_ids: vec![from_id.to_string()],
                ..Default::default()
            }),
            "module" => {
                let project_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_project->project.* AS p FROM ONLY type::record($mid)",
                        "mid",
                        from_id.to_string(),
                        "p",
                    )
                    .await?;

                // Walk up belongs_to_module chain to collect all ancestor modules
                let mut module_ids = vec![from_id.to_string()];
                let mut current = from_id.to_string();
                loop {
                    let parents = self
                        .record_ids_from_query(
                            "SELECT ->belongs_to_module->module.* AS m FROM ONLY type::record($mid)",
                            "mid",
                            current.clone(),
                            "m",
                        )
                        .await?;
                    if parents.is_empty() {
                        break;
                    }
                    let parent = parents.into_iter().next().unwrap();
                    if module_ids.contains(&parent) {
                        break; // cycle guard
                    }
                    module_ids.push(parent.clone());
                    current = parent;
                }

                Ok(StructuralAncestry {
                    project_ids,
                    module_ids,
                    ..Default::default()
                })
            }
            "task" => {
                let project_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_project->project.* AS p FROM ONLY type::record($tid)",
                        "tid",
                        from_id.to_string(),
                        "p",
                    )
                    .await?;
                let module_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_module->module.* AS m FROM ONLY type::record($tid)",
                        "tid",
                        from_id.to_string(),
                        "m",
                    )
                    .await?;
                Ok(StructuralAncestry {
                    project_ids,
                    module_ids,
                    task_ids: vec![from_id.to_string()],
                    epic_ids: vec![],
                })
            }
            "epic" => {
                let project_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_project->project.* AS p FROM ONLY type::record($eid)",
                        "eid",
                        from_id.to_string(),
                        "p",
                    )
                    .await?;
                Ok(StructuralAncestry {
                    project_ids,
                    epic_ids: vec![from_id.to_string()],
                    ..Default::default()
                })
            }
            "file" => {
                let project_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_project->project.* AS p FROM ONLY type::record($fid)",
                        "fid",
                        from_id.to_string(),
                        "p",
                    )
                    .await?;
                let module_ids = self
                    .record_ids_from_query(
                        "SELECT ->belongs_to_module->module.* AS m FROM ONLY type::record($fid)",
                        "fid",
                        from_id.to_string(),
                        "m",
                    )
                    .await?;

                Ok(StructuralAncestry {
                    project_ids,
                    module_ids,
                    ..Default::default()
                })
            }
            _ => Err(anyhow::anyhow!(
                "resolve_structural_ancestry: from_id must be project, module, task, epic, or file; got {:?}",
                table
            )),
        }
    }

    /// Returns all ids from a query result array.
    pub(crate) async fn record_ids_from_query(
        &self,
        sql: &str,
        bind_key: &str,
        bind_val: String,
        result_alias: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut response = self
            .client
            .query(sql)
            .bind((bind_key.to_string(), bind_val))
            .await?;
        let row: Option<surrealdb::types::Value> = response.take(0)?;
        let row = match row {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };
        let json = surreal_to_json(row);
        let mut ids = Vec::new();
        if let Some(arr) = json.get(result_alias).and_then(|v| v.as_array()) {
            for obj in arr {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    ids.push(id.to_string());
                }
            }
        }
        Ok(ids)
    }
}
