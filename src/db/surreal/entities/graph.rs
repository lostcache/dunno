use crate::db::surreal::util::{is_missing_table_error, surreal_to_json};
use crate::db::surreal::DB;

impl DB {
    /// Returns all nodes and edges as a Cytoscape-compatible JSON structure.
    pub async fn get_graph_data(&self) -> anyhow::Result<serde_json::Value> {
        let mut elements: Vec<serde_json::Value> = Vec::new();

        // (table, label_field)
        let node_tables: &[(&str, &str)] = &[
            ("project", "name"),
            ("module", "name"),
            ("submodule", "name"),
            ("file", "name"),
            ("task", "name"),
            ("todo_item", "content"),
            ("context", "type"),
            ("user_story", "title"),
            ("epic", "title"),
            ("persona", "name"),
            ("workflow", "name"),
        ];

        for (table, label_field) in node_tables {
            let records: Vec<surrealdb::types::Value> = match self.client.select(*table).await {
                Ok(v) => v,
                Err(e) if is_missing_table_error(&e) => vec![],
                Err(e) => return Err(e.into()),
            };
            for record in records {
                let json = surreal_to_json(record);
                let id = json
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let label = json
                    .get(*label_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();

                let mut data = serde_json::Map::new();
                data.insert("id".to_string(), serde_json::Value::String(id));
                data.insert("label".to_string(), serde_json::Value::String(label));
                data.insert(
                    "node_type".to_string(),
                    serde_json::Value::String(table.to_string()),
                );
                if let serde_json::Value::Object(obj) = &json {
                    for (k, v) in obj {
                        if k != "id" {
                            data.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }
                }
                elements.push(serde_json::json!({ "data": data }));
            }
        }

        let node_ids: std::collections::HashSet<String> = elements
            .iter()
            .filter_map(|e| e["data"]["id"].as_str().map(|s| s.to_string()))
            .collect();

        let edge_tables = [
            "contains",
            "has_task",
            "belongs_to_project",
            "belongs_to_module",
            "belongs_to_submodule",
            "has_context",
            "has_todo",
            "has_user_story",
            "belongs_to_story",
            "has_module",
            "has_submodule",
            "belongs_to_user_story",
            "has_epic",
            "belongs_to_epic",
            "has_persona",
            "has_workflow",
        ];

        for edge_table in &edge_tables {
            let sql = format!("SELECT in, out FROM {}", edge_table);
            let mut response = match self.client.query(&sql).await {
                Ok(r) => r,
                Err(e) if is_missing_table_error(&e) => continue,
                Err(e) => return Err(e.into()),
            };
            let rows: Vec<surrealdb::types::Value> = match response.take(0) {
                Ok(v) => v,
                Err(_) => vec![],
            };
            for row in rows {
                let json = surreal_to_json(row);
                let source = json
                    .get("in")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let target = json
                    .get("out")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if source.is_empty() || target.is_empty() {
                    continue;
                }
                if !node_ids.contains(&source) || !node_ids.contains(&target) {
                    continue;
                }
                elements.push(serde_json::json!({
                    "data": {
                        "id": format!("{}_{}_{}", edge_table, source, target),
                        "source": source,
                        "target": target,
                        "edge_type": edge_table,
                    }
                }));
            }
        }

        Ok(serde_json::json!({ "elements": elements }))
    }

    /// Returns nodes and edges for a single project as a Cytoscape-compatible JSON structure.
    pub async fn get_graph_data_by_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let mut relevant_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        relevant_ids.insert(project_id.to_string());

        // belongs_to_project WHERE out = pid → in values
        let sql = format!(
            "SELECT in FROM belongs_to_project WHERE out = {}",
            project_id
        );
        if let Ok(mut res) = self.client.query(&sql).await {
            let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
            for row in rows {
                let json = surreal_to_json(row);
                if let Some(id) = json.get("in").and_then(|v| v.as_str()) {
                    relevant_ids.insert(id.to_string());
                }
            }
        }

        // has_todo WHERE in = pid → out values (todo_items)
        let sql = format!("SELECT out FROM has_todo WHERE in = {}", project_id);
        if let Ok(mut res) = self.client.query(&sql).await {
            let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
            for row in rows {
                let json = surreal_to_json(row);
                if let Some(id) = json.get("out").and_then(|v| v.as_str()) {
                    relevant_ids.insert(id.to_string());
                }
            }
        }

        // contains WHERE in = pid → out values (modules)
        let sql = format!("SELECT out FROM contains WHERE in = {}", project_id);
        let mut module_ids: Vec<String> = Vec::new();
        if let Ok(mut res) = self.client.query(&sql).await {
            let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
            for row in rows {
                let json = surreal_to_json(row);
                if let Some(id) = json.get("out").and_then(|v| v.as_str()) {
                    module_ids.push(id.to_string());
                    relevant_ids.insert(id.to_string());
                }
            }
        }

        // contains WHERE in IN [module_ids] → out values (submodules/files under modules)
        let mut submodule_ids: Vec<String> = Vec::new();
        if !module_ids.is_empty() {
            let ids_list = module_ids
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!("SELECT out FROM contains WHERE in IN [{}]", ids_list);
            if let Ok(mut res) = self.client.query(&sql).await {
                let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
                for row in rows {
                    let json = surreal_to_json(row);
                    if let Some(id) = json.get("out").and_then(|v| v.as_str()) {
                        if id.starts_with("submodule:") {
                            submodule_ids.push(id.to_string());
                        }
                        relevant_ids.insert(id.to_string());
                    }
                }
            }
        }

        // contains WHERE in IN [submodule_ids] → out values (files under submodules)
        if !submodule_ids.is_empty() {
            let ids_list = submodule_ids
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!("SELECT out FROM contains WHERE in IN [{}]", ids_list);
            if let Ok(mut res) = self.client.query(&sql).await {
                let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
                for row in rows {
                    let json = surreal_to_json(row);
                    if let Some(id) = json.get("out").and_then(|v| v.as_str()) {
                        relevant_ids.insert(id.to_string());
                    }
                }
            }
        }

        let node_tables: &[(&str, &str)] = &[
            ("project", "name"),
            ("module", "name"),
            ("submodule", "name"),
            ("file", "name"),
            ("task", "name"),
            ("todo_item", "content"),
            ("context", "type"),
            ("user_story", "title"),
            ("epic", "title"),
            ("persona", "name"),
            ("workflow", "name"),
        ];

        let mut elements: Vec<serde_json::Value> = Vec::new();

        for (table, label_field) in node_tables {
            let records: Vec<surrealdb::types::Value> = match self.client.select(*table).await {
                Ok(v) => v,
                Err(e) if is_missing_table_error(&e) => vec![],
                Err(e) => return Err(e.into()),
            };
            for record in records {
                let json = surreal_to_json(record);
                let id = json
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !relevant_ids.contains(&id) {
                    continue;
                }
                let label = json
                    .get(*label_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();

                let mut data = serde_json::Map::new();
                data.insert("id".to_string(), serde_json::Value::String(id));
                data.insert("label".to_string(), serde_json::Value::String(label));
                data.insert(
                    "node_type".to_string(),
                    serde_json::Value::String(table.to_string()),
                );
                if let serde_json::Value::Object(obj) = &json {
                    for (k, v) in obj {
                        if k != "id" {
                            data.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }
                }
                elements.push(serde_json::json!({ "data": data }));
            }
        }

        let node_ids: std::collections::HashSet<String> = elements
            .iter()
            .filter_map(|e| e["data"]["id"].as_str().map(|s| s.to_string()))
            .collect();

        let edge_tables = [
            "contains",
            "has_task",
            "belongs_to_project",
            "belongs_to_module",
            "belongs_to_submodule",
            "has_context",
            "has_todo",
            "has_user_story",
            "belongs_to_story",
            "has_module",
            "has_submodule",
            "belongs_to_user_story",
            "has_epic",
            "belongs_to_epic",
            "has_persona",
            "has_workflow",
        ];

        for edge_table in &edge_tables {
            let sql = format!("SELECT in, out FROM {}", edge_table);
            let mut response = match self.client.query(&sql).await {
                Ok(r) => r,
                Err(e) if is_missing_table_error(&e) => continue,
                Err(e) => return Err(e.into()),
            };
            let rows: Vec<surrealdb::types::Value> = match response.take(0) {
                Ok(v) => v,
                Err(_) => vec![],
            };
            for row in rows {
                let json = surreal_to_json(row);
                let source = json
                    .get("in")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let target = json
                    .get("out")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if source.is_empty() || target.is_empty() {
                    continue;
                }
                if !node_ids.contains(&source) || !node_ids.contains(&target) {
                    continue;
                }
                elements.push(serde_json::json!({
                    "data": {
                        "id": format!("{}_{}_{}", edge_table, source, target),
                        "source": source,
                        "target": target,
                        "edge_type": edge_table,
                    }
                }));
            }
        }

        Ok(serde_json::json!({ "elements": elements }))
    }
}
