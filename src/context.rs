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
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let tags = db.list_category_tags().await?;
    let edges = db.list_edges().await?;
    let neighbors = build_neighbors(edges);
    let node_map = build_node_map(db).await?;
    let seed_ids = collect_seed_ids(&tokens, &tags, &node_map);

    if seed_ids.is_empty() {
        return Ok(Vec::new());
    }

    let visited = traverse_graph(seed_ids.clone(), &neighbors, MAX_HOPS);
    let mut scored: Vec<(i32, String)> = visited
        .into_iter()
        .filter_map(|id| {
            node_map
                .get(&id)
                .map(|node| (node_score(node, &tokens, seed_ids.contains(&id)), id))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    Ok(scored
        .into_iter()
        .take(MAX_RESULTS)
        .filter_map(|(_, id)| node_map.get(&id).cloned())
        .collect())
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

async fn build_node_map(db: &DB) -> Result<HashMap<String, Value>> {
    let mut node_map: HashMap<String, Value> = HashMap::new();
    for item in db.list_mistakes().await? {
        insert_node(
            &mut node_map,
            item.id.clone(),
            serde_json::to_value(item)?,
            "mistake",
        );
    }
    for item in db.list_style_rules().await? {
        insert_node(
            &mut node_map,
            item.id.clone(),
            serde_json::to_value(item)?,
            "style_rule",
        );
    }
    for item in db.list_skills().await? {
        insert_node(
            &mut node_map,
            item.id.clone(),
            serde_json::to_value(item)?,
            "skill",
        );
    }
    Ok(node_map)
}

fn insert_node(
    node_map: &mut HashMap<String, Value>,
    id: Option<String>,
    mut value: Value,
    node_type: &str,
) {
    if let Some(id) = id {
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "node_type".to_string(),
                Value::String(node_type.to_string()),
            );
        }
        node_map.insert(id, value);
    }
}

fn build_neighbors(edges: Vec<crate::models::KnowledgeEdge>) -> HashMap<String, Vec<String>> {
    let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        neighbors
            .entry(edge.from_id.clone())
            .or_default()
            .push(edge.to_id.clone());
        neighbors.entry(edge.to_id).or_default().push(edge.from_id);
    }
    neighbors
}

fn collect_seed_ids(
    tokens: &[String],
    tags: &[crate::models::CategoryTag],
    node_map: &HashMap<String, Value>,
) -> HashSet<String> {
    let mut seed_ids: HashSet<String> = HashSet::new();
    for tag in tags {
        if let Some(tag_id) = &tag.id {
            let normalized = tag.normalized.to_lowercase();
            let display_name = tag.name.to_lowercase();
            let matches = tokens.iter().any(|token| {
                normalized == *token
                    || display_name == *token
                    || normalized.contains(token)
                    || token.contains(&normalized)
            });
            if matches {
                seed_ids.insert(tag_id.clone());
            }
        }
    }

    for (node_id, value) in node_map {
        if node_matches_tokens(value, tokens) {
            seed_ids.insert(node_id.clone());
        }
    }
    seed_ids
}

fn traverse_graph(
    seed_ids: HashSet<String>,
    neighbors: &HashMap<String, Vec<String>>,
    max_hops: usize,
) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    for seed in seed_ids {
        visited.insert(seed.clone());
        queue.push_back((seed, 0));
    }

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_hops {
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
    visited
}

fn node_matches_tokens(node: &Value, tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return true;
    }

    let haystack = node.to_string().to_lowercase();
    tokens.iter().any(|t| haystack.contains(t))
}

fn node_score(node: &Value, tokens: &[String], is_seed: bool) -> i32 {
    let haystack = node.to_string().to_lowercase();
    let token_hits = tokens
        .iter()
        .filter(|token| haystack.contains(*token))
        .count() as i32;
    let seed_bonus = if is_seed { 100 } else { 0 };
    seed_bonus + token_hits
}
