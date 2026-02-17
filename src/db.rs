use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;
use anyhow::Result;
use crate::models::{Mistake, StyleRule, Skill};
use surrealdb::types::Value; 
use serde_json::to_value as to_json_value;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct DB {
    client: Surreal<Any>,
}

impl DB {
    pub async fn new(url: &str) -> Result<Self> {
        let client = connect(url).await?;
        client.use_ns("lazydev").use_db("lazydev").await?;
        Ok(Self { client })
    }

    // Mistake operations
    pub async fn create_mistake(&self, mistake: &Mistake) -> Result<Mistake> {
        let json = to_json_value(mistake)?;
        let value = json_to_surreal(json);
        
        let created: Option<Value> = self.client
            .create("mistake")
            .content(value)
            .await?;
            
        if let Some(val) = created {
             let json = surreal_to_json(val);
             let mistake: Mistake = serde_json::from_value(json)?;
             Ok(mistake)
        } else {
            panic!("Failed to create mistake");
        }
    }

    pub async fn get_mistake(&self, id: &str) -> Result<Option<Mistake>> {
        let key = id.splitn(2, ':').nth(1).unwrap_or(id);
        
        // Use manual query if select fails to find ID?
        // Let's stick to .select and fix the ID if needed.
        // But for now, try to debug why .select returns None.
        // It's possible that when we use .content(value), if value has "id": null,
        // SurrealDB creates a random ID.
        // The returned ID is e.g. "mistake:abc".
        // When we call .select(("mistake", "abc")), it should work.
        
        let fetched: Option<Value> = self.client.select(("mistake", key)).await?;
        
        if let Some(val) = fetched {
             let json = surreal_to_json(val);
             let mistake: Mistake = serde_json::from_value(json)?;
             Ok(Some(mistake))
        } else {
             Ok(None)
        }
    }

    // StyleRule operations
    pub async fn create_style_rule(&self, rule: &StyleRule) -> Result<StyleRule> {
        let json = to_json_value(rule)?;
        let value = json_to_surreal(json);
        
        let created: Option<Value> = self.client
            .create("style_rule")
            .content(value)
            .await?;
            
        if let Some(val) = created {
             let json = surreal_to_json(val);
             let rule: StyleRule = serde_json::from_value(json)?;
             Ok(rule)
        } else {
            panic!("Failed to create style rule");
        }
    }

    // Skill operations
    pub async fn create_skill(&self, skill: &Skill) -> Result<Skill> {
        let json = to_json_value(skill)?;
        let value = json_to_surreal(json);
        
        let created: Option<Value> = self.client
            .create("skill")
            .content(value)
            .await?;
            
        if let Some(val) = created {
             let json = surreal_to_json(val);
             let skill: Skill = serde_json::from_value(json)?;
             Ok(skill)
        } else {
            panic!("Failed to create skill");
        }
    }
}

fn json_to_surreal(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Value::Number(i.into()) }
            else { Value::Number(n.as_f64().unwrap_or(0.0).into()) }
        },
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(a) => {
            Value::Array(a.into_iter().map(json_to_surreal).collect::<Vec<_>>().into())
        },
        serde_json::Value::Object(o) => {
            let mut map = BTreeMap::new();
            for (k, v) in o {
                // If key is "id" and value is null, skip it?
                // If we include "id": null, SurrealDB might try to use "null" as ID?
                if k == "id" && v.is_null() {
                    continue;
                }
                map.insert(k, json_to_surreal(v));
            }
            Value::Object(map.into())
        }
    }
}

fn surreal_to_json(val: Value) -> serde_json::Value {
    match val {
        Value::None | Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => {
            let s = n.to_string();
            if let Ok(i) = s.parse::<i64>() { i.into() }
            else { s.parse::<f64>().unwrap_or(0.0).into() }
        },
        Value::String(s) => s.into(),
        Value::Array(a) => {
             serde_json::Value::Array(a.into_iter().map(surreal_to_json).collect())
        },
        Value::Object(o) => {
             let mut map = serde_json::Map::new();
             for (k, v) in o {
                 map.insert(k, surreal_to_json(v));
             }
             serde_json::Value::Object(map)
        },
        Value::RecordId(t) => {
            let key_json = serde_json::to_value(&t.key).unwrap_or(serde_json::Value::Null);
            let key_str = match key_json {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                _ => format!("{:?}", t.key),
            };
            serde_json::Value::String(format!("{}:{}", t.table, key_str))
        },
        _ => serde_json::Value::Null,
    }
}
