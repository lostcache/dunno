use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Result;
use serde_json::Value;

use crate::db::DB;
use crate::vector_db::VectorDB;

const MAX_HOPS: usize = 2;
const MAX_RESULTS: usize = 25;

/// Retrieves graph-derived context for a natural language query.
pub async fn get_context(query: String, db: &DB, _vector_db: &VectorDB) -> Result<Vec<Value>> {
    let tokens = tokenize(&query);

    let tags = db.list_category_tags().await?;
    let edges = db.list_edges().await?;

    let mut node_map: HashMap<String, Value> = HashMap::new();
    for item in db.list_mistakes().await? {
        if let Some(id) = item.id.clone() {
            let mut value = serde_json::to_value(item)?;
            if let Some(obj) = value.as_object_mut() {
                obj.insert("node_type".to_string(), Value::String("mistake".to_string()));
            }
            node_map.insert(id, value);
        }
    }
    for item in db.list_style_rules().await? {
        if let Some(id) = item.id.clone() {
            let mut value = serde_json::to_value(item)?;
            if let Some(obj) = value.as_object_mut() {
                obj.insert("node_type".to_string(), Value::String("style_rule".to_string()));
            }
            node_map.insert(id, value);
        }
    }
    for item in db.list_skills().await? {
        if let Some(id) = item.id.clone() {
            let mut value = serde_json::to_value(item)?;
            if let Some(obj) = value.as_object_mut() {
                obj.insert("node_type".to_string(), Value::String("skill".to_string()));
            }
            node_map.insert(id, value);
        }
    }

    let mut seed_ids: HashSet<String> = HashSet::new();
    for tag in &tags {
        if let Some(tag_id) = &tag.id {
            let normalized = tag.normalized.to_lowercase();
            if tokens.iter().any(|t| normalized.contains(t) || t.contains(&normalized)) {
                seed_ids.insert(tag_id.clone());
            }
        }
    }

    if seed_ids.is_empty() {
        for (node_id, value) in &node_map {
            if node_matches_tokens(value, &tokens) {
                seed_ids.insert(node_id.clone());
            }
        }
    }

    let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        neighbors
            .entry(edge.from_id.clone())
            .or_default()
            .push(edge.to_id.clone());
        neighbors.entry(edge.to_id).or_default().push(edge.from_id);
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    for seed in seed_ids {
        visited.insert(seed.clone());
        queue.push_back((seed, 0));
    }

    let mut results: Vec<Value> = Vec::new();
    while let Some((current, depth)) = queue.pop_front() {
        if let Some(node) = node_map.get(&current) {
            results.push(node.clone());
            if results.len() >= MAX_RESULTS {
                break;
            }
        }

        if depth >= MAX_HOPS {
            continue;
        }

        if let Some(next_nodes) = neighbors.get(&current) {
            for next in next_nodes {
                if visited.insert(next.clone()) {
                    queue.push_back((next.clone(), depth + 1));
                }
            }
        }
    }

    Ok(results)
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|s| {
            s.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn node_matches_tokens(node: &Value, tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return true;
    }

    let haystack = node.to_string().to_lowercase();
    tokens.iter().any(|t| haystack.contains(t))
}
