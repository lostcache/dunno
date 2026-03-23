pub mod surreal;
pub use surreal::DB;

pub use surreal::{
    get_epic_context_json as get_epic_context, get_file_context_json as get_file_context,
    get_project_structure_json as get_project_structure,
    get_task_context_json as get_task_context,
};
