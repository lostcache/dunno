//! SurrealDB utility helpers: JSON conversion, error detection, record ID validation.

/// Returns `Ok(())` if `id` has the form `{table}:...`; otherwise returns an error.
pub(crate) fn ensure_record_id(table: &str, id: &str) -> anyhow::Result<()> {
    if id.starts_with(&format!("{table}:")) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Expected record id for table {:?}, got {:?}",
            table,
            id
        ))
    }
}

pub(crate) fn is_missing_table_error(err: &surrealdb::Error) -> bool {
    err.to_string().contains("does not exist")
}

pub(crate) fn json_to_surreal(json: serde_json::Value) -> surrealdb::types::Value {
    match json {
        serde_json::Value::Null => surrealdb::types::Value::Null,
        serde_json::Value::Bool(b) => surrealdb::types::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                surrealdb::types::Value::Number(i.into())
            } else {
                surrealdb::types::Value::Number(n.as_f64().unwrap_or(0.0).into())
            }
        }
        serde_json::Value::String(s) => surrealdb::types::Value::String(s),
        serde_json::Value::Array(a) => surrealdb::types::Value::Array(
            a.into_iter()
                .map(json_to_surreal)
                .collect::<Vec<_>>()
                .into(),
        ),
        serde_json::Value::Object(o) => {
            let mut map = std::collections::BTreeMap::new();
            for (k, v) in o {
                // surreal will populate the id field when left as None while creating a record
                if k == "id" && v.is_null() {
                    continue;
                }
                map.insert(k, json_to_surreal(v));
            }
            surrealdb::types::Value::Object(map.into())
        }
    }
}

pub(crate) fn surreal_to_json(val: surrealdb::types::Value) -> serde_json::Value {
    match val {
        surrealdb::types::Value::None | surrealdb::types::Value::Null => serde_json::Value::Null,
        surrealdb::types::Value::Bool(b) => serde_json::Value::Bool(b),
        surrealdb::types::Value::Number(n) => {
            let s = n.to_string();
            if let Ok(i) = s.parse::<i64>() {
                i.into()
            } else {
                s.parse::<f64>().unwrap_or(0.0).into()
            }
        }
        surrealdb::types::Value::String(s) => s.into(),
        surrealdb::types::Value::Array(a) => {
            serde_json::Value::Array(a.into_iter().map(surreal_to_json).collect())
        }
        surrealdb::types::Value::Object(o) => {
            let mut map = serde_json::Map::new();
            for (k, v) in o {
                map.insert(k, surreal_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        surrealdb::types::Value::RecordId(t) => {
            let key_debug = format!("{:?}", t.key);
            let key_str = if let Some(inner) = key_debug
                .strip_prefix("String(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                inner.to_string()
            } else {
                key_debug
            };
            serde_json::Value::String(format!("{}:{}", t.table, key_str))
        }
        _ => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_to_surreal_skips_null_id_field() {
        let json = serde_json::json!({
            "id": null,
            "name": "example",
            "value": 42
        });
        let value = json_to_surreal(json);
        let as_json = surreal_to_json(value);
        assert_eq!(
            as_json,
            serde_json::json!({
                "name": "example",
                "value": 42
            })
        );
    }

    #[test]
    fn surreal_to_json_round_trips_primitives_and_arrays() {
        let original = serde_json::json!({
            "bool": true,
            "int": 5,
            "list": [1, 2, 3]
        });
        let as_surreal = json_to_surreal(original.clone());
        let round_tripped = surreal_to_json(as_surreal);
        assert_eq!(round_tripped, original);
    }

    #[test]
    fn is_missing_table_error_detects_expected_message() {
        let err = surrealdb::Error::thrown("Table `foo` does not exist".into());
        assert!(is_missing_table_error(&err));

        let other = surrealdb::Error::thrown("Some other error".into());
        assert!(!is_missing_table_error(&other));
    }

    #[test]
    fn ensure_record_id_accepts_expected_prefix() {
        ensure_record_id("project", "project:abc").expect("should accept correct record id");
        ensure_record_id("module", "module:123").expect("should accept correct record id");
    }

    #[test]
    fn ensure_record_id_rejects_wrong_or_missing_prefix() {
        let err = ensure_record_id("project", "module:abc").expect_err("should reject wrong table");
        assert!(err.to_string().contains("Expected record id"));

        let err = ensure_record_id("project", "abc").expect_err("should reject missing prefix");
        assert!(err.to_string().contains("Expected record id"));
    }
}
