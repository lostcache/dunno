

/// Retrieves hierarchical context for a task.
///
/// Traverses: Task <-contains<- Module <-contains<- Project
/// Collects has_context knowledge nodes at each level.
pub async fn get_task_context(task_id: &str, db: &crate::db::DB) -> anyhow::Result<Vec<serde_json::Value>> {
    let sql = r#"
        LET $t = type::record($tid);
        LET $modules = (SELECT <-contains<-module AS m FROM ONLY $t).m;
        LET $module = $modules[0];
        LET $projects = (SELECT <-contains<-project AS p FROM ONLY $module).p;
        LET $project = $projects[0];

        LET $t_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $t);
        LET $m_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $module);
        LET $p_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $project);

        RETURN [$t_ctx, $m_ctx, $p_ctx];
    "#;

    let raw = db
        .query_raw_json(sql, "tid", task_id.to_string(), 8)
        .await?;
    Ok(flatten_context_result(raw))
}

/// Retrieves hierarchical context for a file.
///
/// Traverses: File <-contains<- Submodule (optional) <-contains<- Module <-contains<- Project
pub async fn get_file_context(file_id: &str, db: &crate::db::DB) -> anyhow::Result<Vec<serde_json::Value>> {
    let sql = r#"
        LET $f = type::record($fid);

        -- Try submodule parent first
        LET $sub_parents = (SELECT <-contains<-submodule AS s FROM ONLY $f).s;
        LET $submodule = $sub_parents[0];

        -- Module: either parent of submodule, or direct parent of file
        LET $mod_from_sub = (SELECT <-contains<-module AS m FROM ONLY $submodule).m;
        LET $mod_direct = (SELECT <-contains<-module AS m FROM ONLY $f).m;
        LET $module = IF $mod_from_sub != NONE AND array::len($mod_from_sub) > 0
            THEN $mod_from_sub[0]
            ELSE $mod_direct[0]
        END;

        LET $projects = (SELECT <-contains<-project AS p FROM ONLY $module).p;
        LET $project = $projects[0];

        LET $f_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $f);
        LET $s_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $submodule);
        LET $m_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $module);
        LET $p_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $project);

        RETURN [$f_ctx, $s_ctx, $m_ctx, $p_ctx];
    "#;

    let raw = db
        .query_raw_json(sql, "fid", file_id.to_string(), 12)
        .await?;
    Ok(flatten_context_result(raw))
}

/// Retrieves hierarchical context for a subtask.
///
/// Traverses: Subtask <-contains<- Task <-contains<- Module <-contains<- Project
pub async fn get_subtask_context(subtask_id: &str, db: &crate::db::DB) -> anyhow::Result<Vec<serde_json::Value>> {
    let sql = r#"
        LET $st = type::record($stid);
        LET $tasks = (SELECT <-contains<-task AS t FROM ONLY $st).t;
        LET $task = $tasks[0];
        LET $modules = (SELECT <-contains<-module AS m FROM ONLY $task).m;
        LET $module = $modules[0];
        LET $projects = (SELECT <-contains<-project AS p FROM ONLY $module).p;
        LET $project = $projects[0];

        LET $st_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $st);
        LET $t_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $task);
        LET $m_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $module);
        LET $p_ctx = (SELECT
            ->has_context->mistake.*,
            ->has_context->style_rule.*,
            ->has_context->security_detail.*
        FROM ONLY $project);

        RETURN [$st_ctx, $t_ctx, $m_ctx, $p_ctx];
    "#;

    let raw = db
        .query_raw_json(sql, "stid", subtask_id.to_string(), 11)
        .await?;
    Ok(flatten_context_result(raw))
}

/// Flattens the raw SurrealQL context result into a deduplicated list of
/// knowledge node JSON objects, each tagged with a `node_type` field.
pub(crate) fn flatten_context_result(raw: serde_json::Value) -> Vec<serde_json::Value> {
    let mut nodes = Vec::new();

    let levels = match raw {
        serde_json::Value::Array(arr) => arr,
        _ => return nodes,
    };

    for level in levels {
        let level_obj = match level {
            serde_json::Value::Object(map) => map,
            _ => continue,
        };

        // Find the has_context object (key contains "has_context")
        let ctx_obj = match level_obj.iter().find(|(k, _)| k.contains("has_context")) {
            Some((_, serde_json::Value::Object(m))) => m,
            _ => continue,
        };

        // Iterate over knowledge types in ctx_obj
        for (key, val) in ctx_obj.iter() {
            let node_type = if key.contains("mistake") {
                "mistake"
            } else if key.contains("style_rule") {
                "style_rule"
            } else if key.contains("security_detail") {
                "security_detail"
            } else {
                continue;
            };

            if let serde_json::Value::Array(items) = val {
                for inner in items {
                    match inner {
                        serde_json::Value::Array(nested) => {
                            for item in nested {
                                if let serde_json::Value::Object(_) = item {
                                    let mut node = item.clone();
                                    if let serde_json::Value::Object(ref mut m) = node {
                                        m.insert(
                                            "node_type".to_string(),
                                            serde_json::Value::String(node_type.to_string()),
                                        );
                                    }
                                    nodes.push(node);
                                }
                            }
                        }
                        serde_json::Value::Object(_) => {
                            let mut node = inner.clone();
                            if let serde_json::Value::Object(ref mut m) = node {
                                m.insert(
                                    "node_type".to_string(),
                                    serde_json::Value::String(node_type.to_string()),
                                );
                            }
                            nodes.push(node);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Dedup by id
    nodes.sort_by(|a, b| {
        let id_a = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_b = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        id_a.cmp(id_b)
    });
    nodes.dedup_by(|a, b| {
        let id_a = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_b = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        id_a == id_b
    });

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_context_result() {
        let raw = serde_json::json!([
            {
                "has_context": {
                    "mistake": [[
                        { "id": "mistake:1", "content": "Avoid unwrap" }
                    ]],
                    "style_rule": [],
                    "security_detail": []
                }
            },
            {
                "has_context": {
                    "mistake": [],
                    "style_rule": [[
                        { "id": "style_rule:1", "description": "Use match", "example": "match x {}" }
                    ]],
                    "security_detail": []
                }
            },
            {
                "has_context": {
                    "mistake": [[
                        { "id": "mistake:1", "content": "Avoid unwrap" }
                    ]],
                    "style_rule": [],
                    "security_detail": []
                }
            }
        ]);
        let nodes = flatten_context_result(raw);
        assert_eq!(nodes.len(), 2, "dedup by id should yield 2 unique nodes");
        let mistake_node = nodes.iter().find(|n| n.get("node_type").and_then(|v| v.as_str()) == Some("mistake")).expect("one mistake node");
        assert_eq!(mistake_node["content"], "Avoid unwrap");
        assert_eq!(mistake_node["id"], "mistake:1");
        let style_node = nodes.iter().find(|n| n.get("node_type").and_then(|v| v.as_str()) == Some("style_rule")).expect("one style_rule node");
        assert_eq!(style_node["description"], "Use match");
        assert_eq!(style_node["id"], "style_rule:1");
    }
}
