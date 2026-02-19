use crate::models::{
    CategoryTag, KnowledgeEdge, Mistake, Module, Project, Skill, StyleRule, Task, TodoItem,
};
use anyhow::Result;
use serde_json::to_value as to_json_value;
use std::collections::BTreeMap;
use surrealdb::engine::any::{connect, Any};
use surrealdb::types::Value;
use surrealdb::Surreal;

#[derive(Clone)]
pub struct DB {
    client: Surreal<Any>,
}

impl DB {
    /// Creates a new SurrealDB client and selects the default namespace/database.
    pub async fn new(url: &str) -> Result<Self> {
        let client = connect(url).await?;
        client.use_ns("lazydev").use_db("lazydev").await?;
        Ok(Self { client })
    }

    // --- Project Operations ---

    pub async fn create_project(&self, project: &Project) -> Result<Project> {
        let json = to_json_value(project)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("project").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create project"))
        }
    }

    pub async fn get_project(&self, id: &str) -> Result<Option<Project>> {
        self.get_record("project", id).await
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.list_records("project").await
    }

    // --- Module Operations ---

    pub async fn create_module(&self, module: &Module) -> Result<Module> {
        let json = to_json_value(module)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("module").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create module"))
        }
    }

    pub async fn get_module(&self, id: &str) -> Result<Option<Module>> {
        self.get_record("module", id).await
    }

    pub async fn list_modules(&self) -> Result<Vec<Module>> {
        self.list_records("module").await
    }

    // --- Task Operations ---

    pub async fn create_task(&self, task: &Task) -> Result<Task> {
        let json = to_json_value(task)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("task").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create task"))
        }
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        self.get_record("task", id).await
    }

    pub async fn list_tasks(&self) -> Result<Vec<Task>> {
        self.list_records("task").await
    }

    // --- Todo Operations ---

    pub async fn create_todo(&self, todo: &TodoItem) -> Result<TodoItem> {
        let json = to_json_value(todo)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("todo_item").content(value).await?;
        if let Some(val) = created {
            let json = surreal_to_json(val);
            Ok(serde_json::from_value(json)?)
        } else {
            Err(anyhow::anyhow!("Failed to create todo item"))
        }
    }

    pub async fn get_todo(&self, id: &str) -> Result<Option<TodoItem>> {
        self.get_record("todo_item", id).await
    }

    pub async fn list_todos(&self) -> Result<Vec<TodoItem>> {
        self.list_records("todo_item").await
    }

    // --- Generic Helpers ---

    async fn get_record<T: serde::de::DeserializeOwned>(
        &self,
        table: &str,
        id: &str,
    ) -> Result<Option<T>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<Value> = match self.client.select((table, key)).await {
            Ok(value) => value,
            Err(err) if is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            Ok(Some(serde_json::from_value(json)?))
        } else {
            Ok(None)
        }
    }

    async fn list_records<T: serde::de::DeserializeOwned>(&self, table: &str) -> Result<Vec<T>> {
        let fetched: Vec<Value> = match self.client.select(table).await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Creates a new mistake record.
    pub async fn create_mistake(&self, mistake: &Mistake) -> Result<Mistake> {
        let json = to_json_value(mistake)?;
        let value = json_to_surreal(json);

        let created: Option<Value> = self.client.create("mistake").content(value).await?;

        if let Some(val) = created {
            let json = surreal_to_json(val);
            let mistake: Mistake = serde_json::from_value(json)?;
            Ok(mistake)
        } else {
            Err(anyhow::anyhow!("Failed to create mistake"))
        }
    }

    /// Fetches a mistake by record id.
    pub async fn get_mistake(&self, id: &str) -> Result<Option<Mistake>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<Value> = match self.client.select(("mistake", key)).await {
            Ok(value) => value,
            Err(err) if is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            let mistake: Mistake = serde_json::from_value(json)?;
            Ok(Some(mistake))
        } else {
            Ok(None)
        }
    }

    /// Returns all mistakes.
    pub async fn list_mistakes(&self) -> Result<Vec<Mistake>> {
        let fetched: Vec<Value> = match self.client.select("mistake").await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Creates a new style rule record.
    pub async fn create_style_rule(&self, rule: &StyleRule) -> Result<StyleRule> {
        let json = to_json_value(rule)?;
        let value = json_to_surreal(json);

        let created: Option<Value> = self.client.create("style_rule").content(value).await?;

        if let Some(val) = created {
            let json = surreal_to_json(val);
            let rule: StyleRule = serde_json::from_value(json)?;
            Ok(rule)
        } else {
            Err(anyhow::anyhow!("Failed to create style rule"))
        }
    }

    /// Fetches a style rule by record id.
    pub async fn get_style_rule(&self, id: &str) -> Result<Option<StyleRule>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<Value> = match self.client.select(("style_rule", key)).await {
            Ok(value) => value,
            Err(err) if is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            let rule: StyleRule = serde_json::from_value(json)?;
            Ok(Some(rule))
        } else {
            Ok(None)
        }
    }

    /// Returns all style rules.
    pub async fn list_style_rules(&self) -> Result<Vec<StyleRule>> {
        let fetched: Vec<Value> = match self.client.select("style_rule").await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Creates a new skill record.
    pub async fn create_skill(&self, skill: &Skill) -> Result<Skill> {
        let json = to_json_value(skill)?;
        let value = json_to_surreal(json);

        let created: Option<Value> = self.client.create("skill").content(value).await?;

        if let Some(val) = created {
            let json = surreal_to_json(val);
            let skill: Skill = serde_json::from_value(json)?;
            Ok(skill)
        } else {
            Err(anyhow::anyhow!("Failed to create skill"))
        }
    }

    /// Fetches a skill by record id.
    pub async fn get_skill(&self, id: &str) -> Result<Option<Skill>> {
        let key = id.split_once(':').map(|(_, key)| key).unwrap_or(id);
        let fetched: Option<Value> = match self.client.select(("skill", key)).await {
            Ok(value) => value,
            Err(err) if is_missing_table_error(&err) => None,
            Err(err) => return Err(err.into()),
        };

        if let Some(val) = fetched {
            let json = surreal_to_json(val);
            let skill: Skill = serde_json::from_value(json)?;
            Ok(Some(skill))
        } else {
            Ok(None)
        }
    }

    /// Returns all skills.
    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let fetched: Vec<Value> = match self.client.select("skill").await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Creates or returns an existing normalized category tag.
    pub async fn create_or_get_category_tag(&self, name: &str) -> Result<CategoryTag> {
        let normalized = normalize_tag(name);
        let existing = self.list_category_tags().await?;
        if let Some(found) = existing.into_iter().find(|t| t.normalized == normalized) {
            return Ok(found);
        }

        let tag = CategoryTag {
            id: None,
            name: name.to_string(),
            normalized,
        };

        let json = to_json_value(&tag)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("category_tag").content(value).await?;

        if let Some(val) = created {
            let json = surreal_to_json(val);
            let tag: CategoryTag = serde_json::from_value(json)?;
            Ok(tag)
        } else {
            Err(anyhow::anyhow!("Failed to create category tag"))
        }
    }

    /// Returns all category tags.
    pub async fn list_category_tags(&self) -> Result<Vec<CategoryTag>> {
        let fetched: Vec<Value> = match self.client.select("category_tag").await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Creates a graph edge from one record id to another.
    pub async fn create_edge(
        &self,
        from_id: &str,
        to_id: &str,
        relation: &str,
    ) -> Result<KnowledgeEdge> {
        let existing = self.list_edges().await?;
        if let Some(edge) = existing
            .into_iter()
            .find(|e| e.from_id == from_id && e.to_id == to_id && e.relation == relation)
        {
            return Ok(edge);
        }

        let edge = KnowledgeEdge {
            id: None,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relation: relation.to_string(),
        };

        let json = to_json_value(&edge)?;
        let value = json_to_surreal(json);
        let created: Option<Value> = self.client.create("knowledge_edge").content(value).await?;

        if let Some(val) = created {
            let json = surreal_to_json(val);
            let edge: KnowledgeEdge = serde_json::from_value(json)?;
            Ok(edge)
        } else {
            Err(anyhow::anyhow!("Failed to create knowledge edge"))
        }
    }

    /// Returns all graph edges from a specific node.
    pub async fn get_edges_from(&self, from_id: &str) -> Result<Vec<KnowledgeEdge>> {
        let sql = "SELECT * FROM knowledge_edge WHERE from_id = $from";
        let mut response = self.client.query(sql).bind(("from", from_id.to_string())).await?;
        let values: Vec<Value> = response.take(0)?;
        
        let mut out = Vec::with_capacity(values.len());
        for val in values {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Returns all graph edges.
    pub async fn list_edges(&self) -> Result<Vec<KnowledgeEdge>> {
        let fetched: Vec<Value> = match self.client.select("knowledge_edge").await {
            Ok(values) => values,
            Err(err) if is_missing_table_error(&err) => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let mut out = Vec::with_capacity(fetched.len());
        for val in fetched {
            let json = surreal_to_json(val);
            out.push(serde_json::from_value(json)?);
        }
        Ok(out)
    }

    /// Fetches a knowledge node and maps it to a JSON object.
    pub async fn fetch_knowledge_node_json(&self, id: &str) -> Result<Option<serde_json::Value>> {
        if id.starts_with("mistake:") {
            if let Some(item) = self.get_mistake(id).await? {
                let mut value = serde_json::to_value(item)?;
                if let Some(obj) = value.as_object_mut() {
                    obj.insert(
                        "node_type".to_string(),
                        serde_json::Value::String("mistake".to_string()),
                    );
                }
                return Ok(Some(value));
            }
            return Ok(None);
        }

        if id.starts_with("style_rule:") {
            if let Some(item) = self.get_style_rule(id).await? {
                let mut value = serde_json::to_value(item)?;
                if let Some(obj) = value.as_object_mut() {
                    obj.insert(
                        "node_type".to_string(),
                        serde_json::Value::String("style_rule".to_string()),
                    );
                }
                return Ok(Some(value));
            }
            return Ok(None);
        }

        if id.starts_with("skill:") {
            if let Some(item) = self.get_skill(id).await? {
                let mut value = serde_json::to_value(item)?;
                if let Some(obj) = value.as_object_mut() {
                    obj.insert(
                        "node_type".to_string(),
                        serde_json::Value::String("skill".to_string()),
                    );
                }
                return Ok(Some(value));
            }
            return Ok(None);
        }

        Ok(None)
    }
}

fn normalize_tag(name: &str) -> String {
    name.trim().to_lowercase().replace(' ', "_")
}

fn is_missing_table_error(err: &surrealdb::Error) -> bool {
    err.to_string().contains("does not exist")
}

fn json_to_surreal(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else {
                Value::Number(n.as_f64().unwrap_or(0.0).into())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(a) => Value::Array(
            a.into_iter()
                .map(json_to_surreal)
                .collect::<Vec<_>>()
                .into(),
        ),
        serde_json::Value::Object(o) => {
            let mut map = BTreeMap::new();
            for (k, v) in o {
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
            if let Ok(i) = s.parse::<i64>() {
                i.into()
            } else {
                s.parse::<f64>().unwrap_or(0.0).into()
            }
        }
        Value::String(s) => s.into(),
        Value::Array(a) => serde_json::Value::Array(a.into_iter().map(surreal_to_json).collect()),
        Value::Object(o) => {
            let mut map = serde_json::Map::new();
            for (k, v) in o {
                map.insert(k, surreal_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        Value::RecordId(t) => {
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
