//! Public context API: re-exported from the db layer.
//!
//! This keeps `dunno::context::*` stable while allowing the underlying
//! implementations to live in the database backend modules.

pub use crate::db::{get_epic_context, get_file_context, get_project_structure, get_task_context};
