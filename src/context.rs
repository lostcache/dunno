use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::db::DB;
use crate::vector_db::VectorDB;

/// Retrieves hierarchical context for a specific task.
/// 
/// Traverses: Task -> Module -> Project
/// Collects context nodes (Mistakes, StyleRules, Skills) linked at each level.
pub async fn get_task_context(task_id: &str, db: &DB, _vector_db: &VectorDB) -> Result<Vec<Value>> {
    let mut context_nodes = Vec::new();

    // 1. Fetch the Task
    let task = db
        .get_task(task_id)
        .await?
        .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

    // 2. Fetch the Module
    let module = db
        .get_module(&task.module_id)
        .await?
        .ok_or_else(|| anyhow!("Module not found: {}", task.module_id))?;

    // 3. Fetch the Project
    let project = db
        .get_project(&module.project_id)
        .await?
        .ok_or_else(|| anyhow!("Project not found: {}", module.project_id))?;

    // 4. Collect Context from each level (Task -> Module -> Project)
    // Priority: Task > Module > Project (though we just append all for now)
    
    // Task Context
    if let Some(id) = &task.id {
        let nodes = get_linked_context(id, db).await?;
        context_nodes.extend(nodes);
    }

    // Module Context
    if let Some(id) = &module.id {
        let nodes = get_linked_context(id, db).await?;
        context_nodes.extend(nodes);
    }

    // Project Context
    if let Some(id) = &project.id {
        let nodes = get_linked_context(id, db).await?;
        context_nodes.extend(nodes);
    }

    // Deduplicate by ID
    context_nodes.sort_by(|a, b| {
        let id_a = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_b = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        id_a.cmp(id_b)
    });
    context_nodes.dedup_by(|a, b| {
        let id_a = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_b = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        id_a == id_b
    });

    Ok(context_nodes)
}

/// Helper to fetch all knowledge nodes linked FROM a given node ID.
async fn get_linked_context(from_id: &str, db: &DB) -> Result<Vec<Value>> {
    let edges = db.get_edges_from(from_id).await?;
    let mut nodes = Vec::new();

    for edge in edges {
        // We only care about edges that point to knowledge nodes
        if let Ok(Some(node)) = db.fetch_knowledge_node_json(&edge.to_id).await {
            nodes.push(node);
        }
    }

    Ok(nodes)
}
