//! Shared helper to flatten SurrealDB context query results.

/// Flattens the raw SurrealQL context result into a deduplicated list of
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

        // New schema: level is result of "SELECT ->has_context->context.*" so key is "context", value is array of context records.
        for (_key, level_val) in level_obj.iter().filter(|(k, _)| k.contains("context")) {
            if let serde_json::Value::Array(items) = level_val {
                for inner in items {
                    match inner {
                        serde_json::Value::Array(nested) => {
                            for item in nested {
                                if let serde_json::Value::Object(_) = item {
                                    let node_type = item
                                        .get("type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("context");
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
                            let node_type = inner
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("context");
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
    use super::flatten_context_result;

    #[test]
    fn test_flatten_context_result_unified() {
        let raw = serde_json::json!([
            {
                "context": [[
                    { "id": "context:1", "type": "mistake", "content": "Avoid unwrap" }
                ]]
            },
            {
                "context": [[
                    { "id": "context:2", "type": "style_rule", "description": "Use match", "example": "match x {}" }
                ]]
            },
            {
                "context": [[
                    { "id": "context:1", "type": "mistake", "content": "Avoid unwrap" }
                ]]
            }
        ]);
        let nodes = flatten_context_result(raw);
        assert_eq!(nodes.len(), 2, "dedup by id should yield 2 unique nodes");
        let mistake_node = nodes
            .iter()
            .find(|n| n.get("node_type").and_then(|v| v.as_str()) == Some("mistake"))
            .expect("one mistake node");
        assert_eq!(mistake_node["content"], "Avoid unwrap");
        assert_eq!(mistake_node["id"], "context:1");
        let style_node = nodes
            .iter()
            .find(|n| n.get("node_type").and_then(|v| v.as_str()) == Some("style_rule"))
            .expect("one style_rule node");
        assert_eq!(style_node["description"], "Use match");
        assert_eq!(style_node["id"], "context:2");
    }
}
