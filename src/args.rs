#[derive(clap::Parser, Debug)]
#[command(
    name = "dn",
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

    /// Format output with indentation for better readability.
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Ignore case when matching project names (use with --project).
    #[arg(short = 'i', long, global = true)]
    pub ignore_case: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Add a new knowledge entry.",
        long_about = "Persist one knowledge entry with arbitrary fields and optionally link it to structural nodes.",
        after_help = "Examples:\n  dn add --field type --value mistake --field content --value \"Avoid unwrap\" --field severity --value high\n  dn add --field type --value security --field content --value \"SQL injection risk\" --link-to module:abc\n  dn add --field custom_type --value performance --field content --value \"Use parallel iterators\" --field category --value optimization"
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
        name = "ctx",
        about = "Retrieve coding context for a task, file, or subtask.",
        long_about = "Find context directly linked to a task or file.",
        after_help = "Example:\n  dn ctx --task-id task:123\n  dn ctx --file-id file:456"
    )]
    Context {
        /// The Task ID to retrieve context for.
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["file_id", "epic_id"])]
        task_id: Option<String>,

        /// The File ID to retrieve context for.
        #[arg(long, value_name = "FILE_ID", conflicts_with_all = ["task_id", "epic_id"])]
        file_id: Option<String>,

        /// The Epic ID to retrieve context for.
        #[arg(long, value_name = "EPIC_ID", conflicts_with_all = ["task_id", "file_id"])]
        epic_id: Option<String>,

        /// Retrieve full inherited context from parent nodes (Project, Module, Submodule).
        #[arg(long)]
        full: bool,
    },

    #[command(
        about = "Create an edge between existing nodes.",
        long_about = "Link a source node to one or more target nodes via a named edge.",
        after_help = "Example:\n  dn link --from project:abc --edge contains --to module:def\n  dn link --from project:abc --edge has_todo --to todo_item:1 --to todo_item:2"
    )]
    Link {
        /// Source record ID (e.g. project:abc, task:xyz).
        #[arg(long, value_name = "FROM_ID")]
        from_id: String,
        /// Edge name (e.g. contains, has_task, has_todo, has_context, belongs_to_project, belongs_to_module, belongs_to_task).
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
    #[command(name = "add")]
    Create { name: String, description: String },
    #[command(name = "ls")]
    List,
    #[command(name = "rm")]
    Delete { project_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum ModuleCommands {
    #[command(name = "add")]
    Create {
        /// Project ID(s) to link this module to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PROJECT_ID", conflicts_with = "project")]
        project_ids: Vec<String>,
        /// Project name to link this module to (alternative to --project-ids).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_ids")]
        project: Option<String>,
        name: String,
        description: String,
        /// Optional notes for the module.
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
    },
    #[command(name = "rm")]
    Delete { module_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum SubmoduleCommands {
    #[command(name = "add")]
    Create {
        /// Module ID(s) to link this submodule to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        name: String,
        description: String,
        /// Optional notes for the submodule.
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
        #[arg(long)]
        module_id: Option<String>,
    },
    #[command(name = "rm")]
    Delete { submodule_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum FileCommands {
    #[command(name = "add")]
    Create {
        /// Parent ID(s) (module or submodule). Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PARENT_ID")]
        parent_ids: Vec<String>,
        name: String,
        path: String,
        /// Optional description of the file's purpose.
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
        /// Optional notes for the file.
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
        #[arg(long, conflicts_with = "submodule_id")]
        module_id: Option<String>,
        #[arg(long, conflicts_with = "module_id")]
        submodule_id: Option<String>,
    },
    #[command(name = "rm")]
    Delete { file_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum TaskCommands {
    #[command(name = "add")]
    Create {
        /// Module ID (single). Use with one project_id to link task.
        #[arg(long, value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        /// Project ID (single). Use with one module_id to link task.
        #[arg(long, value_name = "PROJECT_ID", conflicts_with = "project")]
        project_ids: Vec<String>,
        /// Project name (single). Use with one module_id to link task (alternative to --project-ids).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_ids")]
        project: Option<String>,
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
    #[command(name = "rm")]
    Delete { task_id: String },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum UserStoryCommands {
    #[command(name = "add")]
    Create {
        /// Project ID to link this user story to.
        #[arg(long, value_name = "PROJECT_ID", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to link this user story to (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
        /// Epic ID(s) to link this user story to. Optional.
        #[arg(long, value_name = "EPIC_ID")]
        epic_ids: Vec<String>,
        title: String,
        description: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
        #[arg(long, value_name = "EPIC_ID")]
        epic_id: Option<String>,
    },
    #[command(name = "rm")]
    Delete { user_story_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum EpicCommands {
    #[command(name = "add")]
    Create {
        /// Project ID to link this epic to.
        #[arg(long, value_name = "PROJECT_ID", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to link this epic to (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
        title: String,
        description: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
    },
    #[command(name = "rm")]
    Delete { epic_id: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum TodoCommands {
    #[command(name = "add")]
    Create {
        /// Project ID(s) to link this todo to. Repeat for multiple. Omit for freestanding.
        #[arg(long, value_name = "PROJECT_ID", conflicts_with = "project")]
        project_ids: Vec<String>,
        /// Project name to link this todo to (alternative to --project-ids).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_ids")]
        project: Option<String>,
        content: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(long, value_name = "PROJECT_NAME", conflicts_with = "project_id")]
        project: Option<String>,
    },
    #[command(name = "rm")]
    Delete { todo_id: String },
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
        let result =
            Args::try_parse_from(["dn", "ctx", "--task-id", "task:1", "--file-id", "file:2"]);
        assert!(result.is_err(), "expected clap to reject conflicting ids");

        // Single id variants should parse successfully.
        let task_ok = Args::try_parse_from(["dn", "ctx", "--task-id", "task:1"]);
        assert!(task_ok.is_ok());

        let file_ok = Args::try_parse_from(["dn", "ctx", "--file-id", "file:2"]);
        assert!(file_ok.is_ok());
    }

    #[test]
    fn context_command_accepts_full_flag() {
        let args = Args::try_parse_from(["dn", "ctx", "--task-id", "task:123", "--full"])
            .expect("parse full flag");
        if let Commands::Context { full, .. } = args.command {
            assert!(full);
        } else {
            panic!("expected Context command");
        }
    }

    #[test]
    fn add_command_accepts_field_value_pairs() {
        let args = Args::try_parse_from([
            "dn",
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
        let result = Args::try_parse_from(["dn", "add", "--field", "type"]);
        assert!(
            result.is_err(),
            "expected clap to require --value when --field is present"
        );

        // Missing --field
        let result2 = Args::try_parse_from(["dn", "add", "--value", "mistake"]);
        assert!(
            result2.is_err(),
            "expected clap to require --field when --value is present"
        );

        // Both missing
        let result3 = Args::try_parse_from(["dn", "add"]);
        assert!(
            result3.is_err(),
            "expected clap to require --field and --value"
        );
    }

    #[test]
    fn add_command_accepts_field_value_with_link_to() {
        let args = Args::try_parse_from([
            "dn",
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
        let args =
            Args::try_parse_from(["dn", "add", "--field", "content", "--value", "Simple note"]);
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
        let args = Args::try_parse_from(["dn", "task", "rm", "task:abc123"]);
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
        let args = Args::try_parse_from(["dn", "task", "rm"]);
        assert!(args.is_err(), "should require task_id for delete command");
    }

    #[test]
    fn todo_delete_command_accepts_todo_id() {
        let args = Args::try_parse_from(["dn", "todo", "rm", "todo_item:abc123"]);
        assert!(args.is_ok(), "should parse todo delete command");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::Delete { todo_id } = command {
                assert_eq!(todo_id, "todo_item:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Todo command");
        }
    }

    #[test]
    fn todo_delete_command_requires_todo_id() {
        let args = Args::try_parse_from(["dn", "todo", "rm"]);
        assert!(args.is_err(), "should require todo_id for delete command");
    }

    #[test]
    fn pretty_flag_defaults_to_false() {
        let args = Args::try_parse_from(["dn", "config", "show"]);
        assert!(args.is_ok(), "should parse config show command");
        assert!(!args.unwrap().pretty, "pretty should default to false");
    }

    #[test]
    fn pretty_flag_can_be_set_true() {
        let args = Args::try_parse_from(["dn", "--pretty", "config", "show"]);
        assert!(args.is_ok(), "should parse with --pretty flag");
        assert!(
            args.unwrap().pretty,
            "pretty should be true when flag is provided"
        );
    }

    #[test]
    fn pretty_flag_works_with_any_command() {
        let args = Args::try_parse_from(["dn", "--pretty", "task", "ls"]);
        assert!(args.is_ok(), "should parse --pretty with task list");
        assert!(args.unwrap().pretty, "pretty should be true");

        let args2 = Args::try_parse_from([
            "dn", "--pretty", "add", "--field", "type", "--value", "test",
        ]);
        assert!(args2.is_ok(), "should parse --pretty with add command");
        assert!(args2.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_backend_flag() {
        let args = Args::try_parse_from(["dn", "--backend", "cloud", "--pretty", "config", "show"]);
        assert!(args.is_ok(), "should parse both --backend and --pretty");
        let parsed = args.unwrap();
        assert_eq!(parsed.backend, Some("cloud".to_string()));
        assert!(parsed.pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_context_command() {
        let args = Args::try_parse_from(["dn", "--pretty", "ctx", "--task-id", "task:abc123"]);
        assert!(args.is_ok(), "should parse --pretty with context command");
        assert!(args.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_epic_commands() {
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "epic",
            "ls",
            "--project-id",
            "project:abc",
        ]);
        assert!(args.is_ok(), "should parse --pretty with epic list");
        assert!(args.unwrap().pretty, "pretty should be true");

        let args2 = Args::try_parse_from([
            "dn",
            "--pretty",
            "epic",
            "add",
            "--project-id",
            "project:abc",
            "Title",
            "Description",
        ]);
        assert!(args2.is_ok(), "should parse --pretty with epic create");
        assert!(args2.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_user_story_commands() {
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "user-story",
            "ls",
            "--project-id",
            "project:abc",
        ]);
        assert!(args.is_ok(), "should parse --pretty with user-story list");
        assert!(args.unwrap().pretty, "pretty should be true");

        let args2 = Args::try_parse_from([
            "dn",
            "--pretty",
            "user-story",
            "add",
            "--project-id",
            "project:abc",
            "As a user",
            "I want to",
        ]);
        assert!(
            args2.is_ok(),
            "should parse --pretty with user-story create"
        );
        assert!(args2.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_todo_commands() {
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "todo",
            "ls",
            "--project-id",
            "project:abc",
        ]);
        assert!(args.is_ok(), "should parse --pretty with todo list");
        assert!(args.unwrap().pretty, "pretty should be true");

        let args2 = Args::try_parse_from([
            "dn",
            "--pretty",
            "todo",
            "add",
            "--project-ids",
            "project:abc",
            "Review code",
        ]);
        assert!(args2.is_ok(), "should parse --pretty with todo create");
        assert!(args2.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_file_commands() {
        // Without description (backward compatibility)
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "file",
            "add",
            "--parent-ids",
            "module:abc",
            "main.rs",
            "src/main.rs",
        ]);
        assert!(args.is_ok(), "should parse --pretty with file create");
        assert!(args.unwrap().pretty, "pretty should be true");

        // With description
        let args_with_desc = Args::try_parse_from([
            "dn",
            "--pretty",
            "file",
            "add",
            "--parent-ids",
            "module:abc",
            "main.rs",
            "src/main.rs",
            "CLI entry point",
        ]);
        assert!(
            args_with_desc.is_ok(),
            "should parse file create with description"
        );

        let args2 =
            Args::try_parse_from(["dn", "--pretty", "file", "ls", "--module-id", "module:abc"]);
        assert!(args2.is_ok(), "should parse --pretty with file list");
        assert!(args2.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn pretty_flag_works_with_link_command() {
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "link",
            "--from-id",
            "project:abc",
            "--edge",
            "contains",
            "--to-ids",
            "module:def",
        ]);
        assert!(args.is_ok(), "should parse --pretty with link command");
        assert!(args.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn ignore_case_flag_defaults_to_false() {
        let args = Args::try_parse_from(["dn", "config", "show"]);
        assert!(args.is_ok(), "should parse config show command");
        assert!(
            !args.unwrap().ignore_case,
            "ignore_case should default to false"
        );
    }

    #[test]
    fn ignore_case_flag_can_be_set_true() {
        let args = Args::try_parse_from(["dn", "-i", "config", "show"]);
        assert!(args.is_ok(), "should parse with -i flag");
        assert!(
            args.unwrap().ignore_case,
            "ignore_case should be true with -i"
        );

        let args2 = Args::try_parse_from(["dn", "--ignore-case", "config", "show"]);
        assert!(args2.is_ok(), "should parse with --ignore-case flag");
        assert!(
            args2.unwrap().ignore_case,
            "ignore_case should be true with --ignore-case"
        );
    }

    #[test]
    fn ignore_case_flag_works_with_any_command() {
        let args = Args::try_parse_from(["dn", "-i", "project", "ls"]);
        assert!(args.is_ok(), "should parse -i with project list");
        assert!(args.unwrap().ignore_case, "ignore_case should be true");

        let args2 = Args::try_parse_from(["dn", "--ignore-case", "task", "ls"]);
        assert!(args2.is_ok(), "should parse --ignore-case with task list");
        assert!(args2.unwrap().ignore_case, "ignore_case should be true");
    }

    #[test]
    fn module_create_accepts_project_name() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "Auth",
            "Auth module",
        ]);
        assert!(args.is_ok(), "should parse --project with module create");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Create { project, name, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(name, "Auth");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_create_rejects_both_project_and_project_ids() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "--project-ids",
            "project:abc",
            "Auth",
            "Auth module",
        ]);
        assert!(
            args.is_err(),
            "should reject both --project and --project-ids"
        );
    }

    #[test]
    fn user_story_create_accepts_project_name() {
        let args = Args::try_parse_from([
            "dn",
            "user-story",
            "add",
            "--project",
            "My Project",
            "As a user, I want login",
            "Login feature",
        ]);
        assert!(
            args.is_ok(),
            "should parse --project with user-story create"
        );
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::Create { project, title, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(title, "As a user, I want login");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected UserStory command");
        }
    }

    #[test]
    fn user_story_create_rejects_both_project_and_project_id() {
        let args = Args::try_parse_from([
            "dn",
            "user-story",
            "add",
            "--project",
            "My Project",
            "--project-id",
            "project:abc",
            "Title",
            "Description",
        ]);
        assert!(
            args.is_err(),
            "should reject both --project and --project-id"
        );
    }

    #[test]
    fn epic_create_accepts_project_name() {
        let args = Args::try_parse_from([
            "dn",
            "epic",
            "add",
            "--project",
            "My Project",
            "Auth Epic",
            "Authentication features",
        ]);
        assert!(args.is_ok(), "should parse --project with epic create");
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::Create { project, title, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(title, "Auth Epic");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Epic command");
        }
    }

    #[test]
    fn todo_create_accepts_project_name() {
        let args =
            Args::try_parse_from(["dn", "todo", "add", "--project", "My Project", "Buy milk"]);
        assert!(args.is_ok(), "should parse --project with todo create");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::Create {
                project, content, ..
            } = command
            {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(content, "Buy milk");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Todo command");
        }
    }

    #[test]
    fn todo_list_accepts_project_name() {
        let args = Args::try_parse_from(["dn", "todo", "ls", "--project", "My Project"]);
        assert!(args.is_ok(), "should parse --project with todo list");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::List { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Todo command");
        }
    }

    #[test]
    fn task_create_accepts_project_name() {
        let args = Args::try_parse_from([
            "dn",
            "task",
            "add",
            "--project",
            "My Project",
            "--module-ids",
            "module:abc",
            "Implement login",
            "Add JWT auth",
        ]);
        assert!(args.is_ok(), "should parse --project with task create");
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::Create { project, name, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(name, "Implement login");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Task command");
        }
    }

    #[test]
    fn module_list_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "module", "ls", "--project-id", "project:abc"]);
        assert!(args.is_ok(), "should parse --project-id with module list");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_list_accepts_project_name() {
        let args = Args::try_parse_from(["dn", "module", "ls", "--project", "My Project"]);
        assert!(args.is_ok(), "should parse --project with module list");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::List { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_list_rejects_both_project_and_project_id() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "ls",
            "--project-id",
            "project:abc",
            "--project",
            "My Project",
        ]);
        assert!(
            args.is_err(),
            "should reject both --project-id and --project"
        );
    }

    #[test]
    fn submodule_list_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "submodule", "ls", "--project-id", "project:abc"]);
        assert!(
            args.is_ok(),
            "should parse --project-id with submodule list"
        );
        if let Commands::Submodule { command } = args.unwrap().command {
            if let SubmoduleCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Submodule command");
        }
    }

    #[test]
    fn submodule_list_accepts_module_id() {
        let args = Args::try_parse_from(["dn", "submodule", "ls", "--module-id", "module:abc"]);
        assert!(args.is_ok(), "should parse --module-id with submodule list");
        if let Commands::Submodule { command } = args.unwrap().command {
            if let SubmoduleCommands::List { module_id, .. } = command {
                assert_eq!(module_id, Some("module:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Submodule command");
        }
    }

    #[test]
    fn file_list_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "file", "ls", "--project-id", "project:abc"]);
        assert!(args.is_ok(), "should parse --project-id with file list");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn file_list_accepts_module_id() {
        let args = Args::try_parse_from(["dn", "file", "ls", "--module-id", "module:abc"]);
        assert!(args.is_ok(), "should parse --module-id with file list");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::List { module_id, .. } = command {
                assert_eq!(module_id, Some("module:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn file_list_accepts_submodule_id() {
        let args = Args::try_parse_from(["dn", "file", "ls", "--submodule-id", "submodule:abc"]);
        assert!(args.is_ok(), "should parse --submodule-id with file list");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::List { submodule_id, .. } = command {
                assert_eq!(submodule_id, Some("submodule:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn task_list_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "task", "ls", "--project-id", "project:abc"]);
        assert!(args.is_ok(), "should parse --project-id with task list");
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Task command");
        }
    }

    #[test]
    fn task_list_accepts_project_name() {
        let args = Args::try_parse_from(["dn", "task", "ls", "--project", "My Project"]);
        assert!(args.is_ok(), "should parse --project with task list");
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::List { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Task command");
        }
    }

    #[test]
    fn project_delete_command_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "project", "rm", "project:abc123"]);
        assert!(args.is_ok(), "should parse project delete command");
        if let Commands::Project { command } = args.unwrap().command {
            if let ProjectCommands::Delete { project_id } = command {
                assert_eq!(project_id, "project:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Project command");
        }
    }

    #[test]
    fn module_delete_command_accepts_module_id() {
        let args = Args::try_parse_from(["dn", "module", "rm", "module:abc123"]);
        assert!(args.is_ok(), "should parse module delete command");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Delete { module_id } = command {
                assert_eq!(module_id, "module:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn submodule_delete_command_accepts_submodule_id() {
        let args = Args::try_parse_from(["dn", "submodule", "rm", "submodule:abc123"]);
        assert!(args.is_ok(), "should parse submodule delete command");
        if let Commands::Submodule { command } = args.unwrap().command {
            if let SubmoduleCommands::Delete { submodule_id } = command {
                assert_eq!(submodule_id, "submodule:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Submodule command");
        }
    }

    #[test]
    fn file_delete_command_accepts_file_id() {
        let args = Args::try_parse_from(["dn", "file", "rm", "file:abc123"]);
        assert!(args.is_ok(), "should parse file delete command");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::Delete { file_id } = command {
                assert_eq!(file_id, "file:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn user_story_delete_command_accepts_user_story_id() {
        let args = Args::try_parse_from(["dn", "user-story", "rm", "user_story:abc123"]);
        assert!(args.is_ok(), "should parse user-story delete command");
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::Delete { user_story_id } = command {
                assert_eq!(user_story_id, "user_story:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected UserStory command");
        }
    }

    #[test]
    fn epic_delete_command_accepts_epic_id() {
        let args = Args::try_parse_from(["dn", "epic", "rm", "epic:abc123"]);
        assert!(args.is_ok(), "should parse epic delete command");
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::Delete { epic_id } = command {
                assert_eq!(epic_id, "epic:abc123");
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Epic command");
        }
    }
}
