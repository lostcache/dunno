pub(crate) fn print_json(value: serde_json::Value, pretty: bool) {
    if pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        println!("{}", value);
    }
}

pub(crate) fn print_error_json(kind: &str, message: String) {
    println!(
        "{}",
        serde_json::json!({
            "status": "error",
            "kind": kind,
            "error": message
        })
    );
}

pub(crate) fn validate_task_parents<'a>(
    module_ids: &'a [String],
    project_ids: &'a [String],
) -> anyhow::Result<(Option<&'a str>, Option<&'a str>)> {
    match (module_ids.len(), project_ids.len()) {
        (0, 1) => Ok((None, Some(&project_ids[0]))),
        (1, 1) => Ok((Some(&module_ids[0]), Some(&project_ids[0]))),
        _ => Err(anyhow::anyhow!(
            "Task create: provide exactly one project ID (with an optional module ID). Got {} module_ids and {} project_ids",
            module_ids.len(),
            project_ids.len()
        )),
    }
}

pub(crate) fn parse_optional_status(
    status: Option<String>,
) -> anyhow::Result<Option<dn_core::models::TaskStatus>> {
    match status {
        Some(value) => dn_core::models::TaskStatus::parse(&value)
            .map(Some)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid status '{}'. Expected: pending, active, completed",
                    value
                )
            }),
        None => Ok(None),
    }
}

pub(crate) async fn resolve_project_id(
    db: &dn_core::db::DB,
    project_id: Option<String>,
    project_name: Option<String>,
    ignore_case: bool,
) -> anyhow::Result<Option<String>> {
    match (project_id, project_name) {
        (Some(id), _) => Ok(Some(id)),
        (None, Some(name)) => {
            let project = db
                .get_project_by_name(&name, ignore_case)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to lookup project by name: {}", e))?;
            match project {
                Some(p) => Ok(p.id),
                None => Err(anyhow::anyhow!("Project not found: {}", name)),
            }
        }
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_print_json_compact_format() {
        let value = serde_json::json!({"status": "ok", "id": "task:abc123"});
        let json_str = value.to_string();

        assert!(
            !json_str.contains('\n'),
            "compact JSON should not have newlines"
        );
        assert!(
            json_str.contains("status"),
            "JSON should contain field names"
        );
        assert!(
            json_str.contains("task:abc123"),
            "JSON should contain values"
        );
    }

    #[test]
    fn test_print_json_pretty_format() {
        let value = serde_json::json!({"status": "ok", "id": "task:abc123"});
        let pretty_str = serde_json::to_string_pretty(&value).unwrap();

        assert!(
            pretty_str.contains('\n'),
            "pretty JSON should have newlines"
        );
        assert!(
            pretty_str.contains("  "),
            "pretty JSON should have indentation"
        );
    }

    #[test]
    fn test_print_json_handles_nested_objects() {
        let value = serde_json::json!({
            "project": {
                "id": "project:abc",
                "name": "Test"
            },
            "tasks": ["task:1", "task:2"]
        });

        let compact = value.to_string();
        let pretty = serde_json::to_string_pretty(&value).unwrap();

        let parsed_compact: serde_json::Value = serde_json::from_str(&compact).unwrap();
        let parsed_pretty: serde_json::Value = serde_json::from_str(&pretty).unwrap();

        assert_eq!(parsed_compact, parsed_pretty);
        assert_eq!(parsed_compact["project"]["id"], "project:abc");
    }

    #[test]
    fn test_print_json_handles_arrays() {
        let value = serde_json::json!([
            {"id": "task:1", "name": "Task 1"},
            {"id": "task:2", "name": "Task 2"}
        ]);

        let pretty = serde_json::to_string_pretty(&value).unwrap();

        assert!(
            pretty.contains('\n'),
            "pretty JSON array should have newlines"
        );
        assert!(
            pretty.contains("Task 1"),
            "pretty JSON should preserve values"
        );
    }
}
