#[derive(clap::Parser, Debug)]
#[command(
    name = "dunno",
    author,
    version,
    about = "Capture and retrieve coding knowledge from mistakes, style guides, and security details.",
    long_about = "dunno stores coding knowledge in a graph database and retrieves context via deterministic hierarchy traversal.",
    propagate_version = true
)]
pub struct Args {
    /// Optional storage backend override (`local` or `cloud`).
    #[arg(long, global = true, value_name = "BACKEND")]
    pub backend: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Add a new knowledge entry.",
        long_about = "Persist one knowledge entry with arbitrary fields and optionally link it to structural nodes.",
        after_help = "Examples:\n  dunno add --field type --value mistake --field content --value \"Avoid unwrap\" --field severity --value high\n  dunno add --field type --value security --field content --value \"SQL injection risk\" --link-to module:abc\n  dunno add --field custom_type --value performance --field content --value \"Use parallel iterators\" --field category --value optimization"
    )]
    Add {
        /// Field name. Must be paired with --value. Repeat for multiple fields.
        #[arg(long = "field", value_name = "FIELD_NAME", required = true)]
        field_names: Vec<String>,

        /// Field value. Must be paired with --field. Repeat for multiple fields.
        #[arg(long = "value", value_name = "FIELD_VALUE", required = true)]
        field_values: Vec<String>,

        /// Structural node ID(s) to link this knowledge to. Repeat for multiple.
        #[arg(long, value_name = "LINK_TO")]
        link_to: Vec<String>,
    },

    #[command(about = "Manage projects.")]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    #[command(about = "Manage modules.")]
    Module {
        #[command(subcommand)]
        command: ModuleCommands,
    },

    #[command(about = "Manage submodules.")]
    Submodule {
        #[command(subcommand)]
        command: SubmoduleCommands,
    },

    #[command(about = "Manage files.")]
    File {
        #[command(subcommand)]
        command: FileCommands,
    },

    #[command(about = "Manage tasks.")]
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    #[command(about = "Manage subtasks.")]
    Subtask {
        #[command(subcommand)]
        command: SubtaskCommands,
    },

    #[command(about = "Manage user stories.")]
    UserStory {
        #[command(subcommand)]
        command: UserStoryCommands,
    },

    #[command(about = "Manage epics.")]
    Epic {
        #[command(subcommand)]
        command: EpicCommands,
    },

    #[command(about = "Manage todo items.")]
    Todo {
        #[command(subcommand)]
        command: TodoCommands,
    },

    #[command(about = "Inspect resolved runtime configuration.")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    #[command(
        about = "Retrieve coding context for a task, file, or subtask.",
        long_about = "Find context directly linked to a task, file, or subtask.",
        after_help = "Example:\n  dunno context --task-id task:123\n  dunno context --file-id file:456\n  dunno context --subtask-id subtask:789"
    )]
    Context {
        /// The Task ID to retrieve context for.
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["file_id", "subtask_id", "epic_id"])]
        task_id: Option<String>,

        /// The File ID to retrieve context for.
        #[arg(long, value_name = "FILE_ID", conflicts_with_all = ["task_id", "subtask_id", "epic_id"])]
        file_id: Option<String>,

        /// The Subtask ID to retrieve context for.
        #[arg(long, value_name = "SUBTASK_ID", conflicts_with_all = ["task_id", "file_id", "epic_id"])]
        subtask_id: Option<String>,

        /// The Epic ID to retrieve context for.
        #[arg(long, value_name = "EPIC_ID", conflicts_with_all = ["task_id", "file_id", "subtask_id"])]
        epic_id: Option<String>,
    },

    #[command(
        about = "Create an edge between existing nodes.",
        long_about = "Link a source node to one or more target nodes via a named edge.",
        after_help = "Example:\n  dunno link --from project:abc --edge contains --to module:def\n  dunno link --from project:abc --edge has_todo --to todo_item:1 --to todo_item:2"
    )]
    Link {
        /// Source record ID (e.g. project:abc, task:xyz).
        #[arg(long, value_name = "FROM_ID")]
        from_id: String,
        /// Edge name (e.g. contains, has_task, has_subtask, has_todo, has_context, belongs_to_project, belongs_to_module, belongs_to_task).
        #[arg(long, value_name = "EDGE")]
        edge: String,
        /// Target record ID(s). Repeat for multiple.
        #[arg(long, value_name = "TO_ID")]
        to_ids: Vec<String>,
    },

    #[command(
        about = "Purge the database (DANGER).",
        long_about = "Delete all records from the database. This action is irreversible.",
        hide = true
    )]
    Purge,
}

#[derive(clap::Subcommand, Debug)]
pub enum ProjectCommands {
    Create { name: String, description: String },
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum ModuleCommands {
    Create {
        /// Project ID(s) to link this module to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PROJECT_ID")]
        project_ids: Vec<String>,
        name: String,
        description: String,
    },
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubmoduleCommands {
    Create {
        /// Module ID(s) to link this submodule to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        name: String,
        description: String,
    },
    List {
        #[arg(long)]
        module_id: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum FileCommands {
    Create {
        /// Parent ID(s) (module or submodule). Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PARENT_ID")]
        parent_ids: Vec<String>,
        name: String,
        path: String,
    },
    List {
        #[arg(long, conflicts_with = "submodule_id")]
        module_id: Option<String>,
        #[arg(long, conflicts_with = "module_id")]
        submodule_id: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum TaskCommands {
    Create {
        /// Module ID (single). Use with one project_id to link task.
        #[arg(long, value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        /// Project ID (single). Use with one module_id to link task.
        #[arg(long, value_name = "PROJECT_ID")]
        project_ids: Vec<String>,
        /// User Story ID(s) to link this task to. Optional.
        #[arg(long, value_name = "USER_STORY_ID")]
        user_story_ids: Vec<String>,
        /// Epic ID(s) to link this task to. Optional.
        #[arg(long, value_name = "EPIC_ID")]
        epic_ids: Vec<String>,
        name: String,
        description: String,
    },
    Update {
        task_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(
            long,
            value_name = "STATUS",
            help = "One of: not_started, started, finished"
        )]
        status: Option<String>,
    },
    Delete {
        task_id: String,
    },
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubtaskCommands {
    Create {
        /// Task ID(s) to link this subtask to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "TASK_ID")]
        task_ids: Vec<String>,
        name: String,
        description: String,
    },
    List {
        #[arg(long)]
        task_id: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum UserStoryCommands {
    Create {
        /// Project ID to link this user story to.
        #[arg(long, value_name = "PROJECT_ID")]
        project_id: String,
        /// Epic ID(s) to link this user story to. Optional.
        #[arg(long, value_name = "EPIC_ID")]
        epic_ids: Vec<String>,
        title: String,
        description: String,
    },
    List {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long, value_name = "EPIC_ID")]
        epic_id: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum EpicCommands {
    Create {
        /// Project ID to link this epic to.
        #[arg(long, value_name = "PROJECT_ID")]
        project_id: String,
        title: String,
        description: String,
    },
    List {
        #[arg(long)]
        project_id: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum TodoCommands {
    Create {
        /// Project ID(s) to link this todo to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PROJECT_ID")]
        project_ids: Vec<String>,
        content: String,
    },
    List {
        #[arg(long)]
        project_id: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommands {
    Show,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn context_command_enforces_mutual_exclusion_of_ids() {
        // TASK_ID and FILE_ID together should be rejected by clap.
        let result = Args::try_parse_from([
            "dunno",
            "context",
            "--task-id",
            "task:1",
            "--file-id",
            "file:2",
        ]);
        assert!(result.is_err(), "expected clap to reject conflicting ids");

        // Single id variants should parse successfully.
        let task_ok = Args::try_parse_from(["dunno", "context", "--task-id", "task:1"]);
        assert!(task_ok.is_ok());

        let file_ok = Args::try_parse_from(["dunno", "context", "--file-id", "file:2"]);
        assert!(file_ok.is_ok());

        let subtask_ok = Args::try_parse_from(["dunno", "context", "--subtask-id", "subtask:3"]);
        assert!(subtask_ok.is_ok());
    }

    #[test]
    fn add_command_accepts_field_value_pairs() {
        let args = Args::try_parse_from([
            "dunno",
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Avoid unwrap",
            "--field",
            "severity",
            "--value",
            "high",
        ]);
        assert!(args.is_ok(), "should parse --field and --value flags");
        if let Commands::Add {
            field_names,
            field_values,
            ..
        } = &args.unwrap().command
        {
            assert_eq!(field_names.len(), 3);
            assert_eq!(field_values.len(), 3);
            assert_eq!(field_names[0], "type");
            assert_eq!(field_values[0], "mistake");
            assert_eq!(field_names[1], "content");
            assert_eq!(field_values[1], "Avoid unwrap");
            assert_eq!(field_names[2], "severity");
            assert_eq!(field_values[2], "high");
        } else {
            panic!("expected Add command");
        }
    }

    #[test]
    fn add_command_requires_field_and_value() {
        // Missing --value
        let result = Args::try_parse_from(["dunno", "add", "--field", "type"]);
        assert!(
            result.is_err(),
            "expected clap to require --value when --field is present"
        );

        // Missing --field
        let result2 = Args::try_parse_from(["dunno", "add", "--value", "mistake"]);
        assert!(
            result2.is_err(),
            "expected clap to require --field when --value is present"
        );

        // Both missing
        let result3 = Args::try_parse_from(["dunno", "add"]);
        assert!(
            result3.is_err(),
            "expected clap to require --field and --value"
        );
    }

    #[test]
    fn add_command_accepts_field_value_with_link_to() {
        let args = Args::try_parse_from([
            "dunno",
            "add",
            "--field",
            "type",
            "--value",
            "performance",
            "--field",
            "content",
            "--value",
            "Use iterators",
            "--link-to",
            "project:abc",
            "--link-to",
            "task:def",
        ]);
        assert!(args.is_ok(), "should parse --field/--value with --link-to");
        if let Commands::Add {
            field_names,
            field_values,
            link_to,
            ..
        } = args.unwrap().command
        {
            assert_eq!(field_names.len(), 2);
            assert_eq!(field_values.len(), 2);
            assert_eq!(link_to.len(), 2);
            assert_eq!(link_to[0], "project:abc");
            assert_eq!(link_to[1], "task:def");
        } else {
            panic!("expected Add command");
        }
    }

    #[test]
    fn add_command_single_field_value() {
        let args = Args::try_parse_from([
            "dunno",
            "add",
            "--field",
            "content",
            "--value",
            "Simple note",
        ]);
        assert!(args.is_ok(), "should parse single --field/--value pair");
        if let Commands::Add {
            field_names,
            field_values,
            ..
        } = &args.unwrap().command
        {
            assert_eq!(field_names.len(), 1);
            assert_eq!(field_values.len(), 1);
            assert_eq!(field_names[0], "content");
            assert_eq!(field_values[0], "Simple note");
        } else {
            panic!("expected Add command");
        }
    }

    #[test]
    fn task_delete_command_accepts_task_id() {
        let args = Args::try_parse_from(["dunno", "task", "delete", "task:abc123"]);
        assert!(args.is_ok(), "should parse task delete command");
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::Delete { task_id } = command {
                assert_eq!(task_id, "task:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Task command");
        }
    }

    #[test]
    fn task_delete_command_requires_task_id() {
        let args = Args::try_parse_from(["dunno", "task", "delete"]);
        assert!(args.is_err(), "should require task_id for delete command");
    }
}
