//! JSON <-> Surreal value conversion and error helpers.

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
