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
    #[arg(long, visible_alias = "b", global = true, value_name = "BACKEND")]
    pub backend: Option<String>,

    /// Format output with indentation for better readability.
    #[arg(long, visible_alias = "pp", global = true)]
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
        #[arg(
            short = 'f',
            long = "field",
            value_name = "FIELD_NAME",
            required = true
        )]
        field_names: Vec<String>,

        /// Field value. Must be paired with --field. Repeat for multiple fields.
        #[arg(
            short = 'v',
            long = "value",
            value_name = "FIELD_VALUE",
            required = true
        )]
        field_values: Vec<String>,

        /// Structural node ID(s) to link this knowledge to. Repeat for multiple.
        #[arg(long, visible_alias = "ln", value_name = "LINK_TO")]
        link_to: Vec<String>,
    },

    #[command(
        about = "Manage projects.",
        visible_alias = "proj",
        visible_alias = "prj"
    )]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    #[command(
        about = "Manage modules.",
        visible_alias = "mod",
        visible_alias = "mdl"
    )]
    Module {
        #[command(subcommand)]
        command: ModuleCommands,
    },

    #[command(
        about = "Manage submodules.",
        visible_alias = "submod",
        visible_alias = "sub"
    )]
    Submodule {
        #[command(subcommand)]
        command: SubmoduleCommands,
    },

    #[command(about = "Manage files.", visible_alias = "f", visible_alias = "fi")]
    File {
        #[command(subcommand)]
        command: FileCommands,
    },

    #[command(about = "Manage tasks.", visible_alias = "t", visible_alias = "tk")]
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    #[command(
        about = "Manage user stories.",
        visible_alias = "us",
        visible_alias = "story"
    )]
    UserStory {
        #[command(subcommand)]
        command: UserStoryCommands,
    },

    #[command(about = "Manage epics.", visible_alias = "ep", visible_alias = "e")]
    Epic {
        #[command(subcommand)]
        command: EpicCommands,
    },

    #[command(
        about = "Manage todo items.",
        visible_alias = "td",
        visible_alias = "to"
    )]
    Todo {
        #[command(subcommand)]
        command: TodoCommands,
    },

    #[command(
        about = "Inspect resolved runtime configuration.",
        visible_alias = "cfg",
        visible_alias = "conf"
    )]
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
        #[arg(long, visible_alias = "tid", value_name = "TASK_ID", conflicts_with_all = ["file_id", "epic_id"])]
        task_id: Option<String>,

        /// The File ID to retrieve context for.
        #[arg(long, visible_alias = "fid", value_name = "FILE_ID", conflicts_with_all = ["task_id", "epic_id"])]
        file_id: Option<String>,

        /// The Epic ID to retrieve context for.
        #[arg(long, visible_alias = "eid", value_name = "EPIC_ID", conflicts_with_all = ["task_id", "file_id"])]
        epic_id: Option<String>,

        /// Retrieve full inherited context from parent nodes (Project, Module, Submodule).
        #[arg(long)]
        full: bool,
    },

    #[command(
        about = "Create an edge between existing nodes.",
        visible_alias = "ln",
        long_about = "Link a source node to one or more target nodes via a named edge.",
        after_help = "Example:\n  dn link --from project:abc --edge contains --to module:def\n  dn link --from project:abc --edge has_todo --to todo_item:1 --to todo_item:2"
    )]
    Link {
        /// Source record ID (e.g. project:abc, task:xyz).
        #[arg(short, long, value_name = "FROM_ID")]
        from_id: String,
        /// Edge name (e.g. contains, has_task, has_todo, has_context, belongs_to_project, belongs_to_module, belongs_to_task).
        #[arg(short, long, value_name = "EDGE")]
        edge: String,
        /// Target record ID(s). Repeat for multiple.
        #[arg(short, long, value_name = "TO_ID")]
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
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_ids: Vec<String>,
        /// Project name to link this module to (alternative to --project-ids).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_ids"
        )]
        project: Option<String>,
        name: String,
        description: String,
        /// Optional notes for the module.
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
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
        #[arg(long, visible_alias = "mids", value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        name: String,
        description: String,
        /// Optional notes for the submodule.
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        #[arg(long, visible_alias = "mid")]
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
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        #[arg(long, visible_alias = "mid", conflicts_with = "submodule_id")]
        module_id: Option<String>,
        #[arg(long, visible_alias = "smid", conflicts_with = "module_id")]
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
        #[arg(long, visible_alias = "mids", value_name = "MODULE_ID")]
        module_ids: Vec<String>,
        /// Project ID (single). Use with one module_id to link task.
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_ids: Vec<String>,
        /// Project name (single). Use with one module_id to link task (alternative to --project-ids).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_ids"
        )]
        project: Option<String>,
        /// User Story ID(s) to link this task to. Optional.
        #[arg(long, visible_alias = "usids", value_name = "USER_STORY_ID")]
        user_story_ids: Vec<String>,
        /// Epic ID(s) to link this task to. Optional.
        #[arg(long, visible_alias = "eids", value_name = "EPIC_ID")]
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
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum UserStoryCommands {
    #[command(name = "add")]
    Create {
        /// Project ID to link this user story to.
        #[arg(
            long,
            visible_alias = "pid",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_id: Option<String>,
        /// Project name to link this user story to (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        /// Epic ID(s) to link this user story to. Optional.
        #[arg(long, visible_alias = "eids", value_name = "EPIC_ID")]
        epic_ids: Vec<String>,
        title: String,
        description: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        #[arg(long, visible_alias = "eid", value_name = "EPIC_ID")]
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
        #[arg(
            long,
            visible_alias = "pid",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_id: Option<String>,
        /// Project name to link this epic to (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
        project: Option<String>,
        title: String,
        description: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
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
        #[arg(
            long,
            visible_alias = "pids",
            value_name = "PROJECT_ID",
            conflicts_with = "project"
        )]
        project_ids: Vec<String>,
        /// Project name to link this todo to (alternative to --project-ids).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_ids"
        )]
        project: Option<String>,
        content: String,
    },
    #[command(name = "ls")]
    List {
        #[arg(long, visible_alias = "pid", conflicts_with = "project")]
        project_id: Option<String>,
        /// Project name to filter by (alternative to --project-id).
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT_NAME",
            conflicts_with = "project_id"
        )]
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

    // Short flag tests
    #[test]
    fn add_command_accepts_short_field_value_flags() {
        let args = Args::try_parse_from([
            "dn",
            "add",
            "-f",
            "type",
            "-v",
            "mistake",
            "-f",
            "content",
            "-v",
            "Short flag test",
        ]);
        assert!(args.is_ok(), "should parse -f and -v short flags");
        if let Commands::Add {
            field_names,
            field_values,
            ..
        } = args.unwrap().command
        {
            assert_eq!(field_names.len(), 2);
            assert_eq!(field_values.len(), 2);
            assert_eq!(field_names[0], "type");
            assert_eq!(field_values[0], "mistake");
            assert_eq!(field_names[1], "content");
            assert_eq!(field_values[1], "Short flag test");
        } else {
            panic!("expected Add command");
        }
    }

    #[test]
    fn add_command_accepts_short_link_to_flag() {
        let args = Args::try_parse_from([
            "dn",
            "add",
            "-f",
            "type",
            "-v",
            "test",
            "--ln",
            "project:abc",
            "--ln",
            "task:def",
        ]);
        assert!(args.is_ok(), "should parse --ln alias for --link-to");
        if let Commands::Add { link_to, .. } = args.unwrap().command {
            assert_eq!(link_to.len(), 2);
            assert_eq!(link_to[0], "project:abc");
            assert_eq!(link_to[1], "task:def");
        } else {
            panic!("expected Add command");
        }
    }

    #[test]
    fn link_command_accepts_short_flags() {
        let args = Args::try_parse_from([
            "dn",
            "link",
            "-f",
            "project:abc",
            "-e",
            "contains",
            "-t",
            "module:def",
            "-t",
            "module:ghi",
        ]);
        assert!(args.is_ok(), "should parse -f, -e, -t short flags for link");
        if let Commands::Link {
            from_id,
            edge,
            to_ids,
        } = args.unwrap().command
        {
            assert_eq!(from_id, "project:abc");
            assert_eq!(edge, "contains");
            assert_eq!(to_ids.len(), 2);
            assert_eq!(to_ids[0], "module:def");
            assert_eq!(to_ids[1], "module:ghi");
        } else {
            panic!("expected Link command");
        }
    }

    #[test]
    fn context_command_accepts_short_id_flags() {
        // Test --tid alias
        let args = Args::try_parse_from(["dn", "ctx", "--tid", "task:abc"]);
        assert!(args.is_ok(), "should parse --tid alias");
        if let Commands::Context { task_id, .. } = args.unwrap().command {
            assert_eq!(task_id, Some("task:abc".to_string()));
        } else {
            panic!("expected Context command");
        }

        // Test --fid alias
        let args = Args::try_parse_from(["dn", "ctx", "--fid", "file:def"]);
        assert!(args.is_ok(), "should parse --fid alias");
        if let Commands::Context { file_id, .. } = args.unwrap().command {
            assert_eq!(file_id, Some("file:def".to_string()));
        } else {
            panic!("expected Context command");
        }

        // Test --eid alias
        let args = Args::try_parse_from(["dn", "ctx", "--eid", "epic:ghi"]);
        assert!(args.is_ok(), "should parse --eid alias");
        if let Commands::Context { epic_id, .. } = args.unwrap().command {
            assert_eq!(epic_id, Some("epic:ghi".to_string()));
        } else {
            panic!("expected Context command");
        }
    }

    #[test]
    fn module_commands_accepts_short_project_flags() {
        // Test module add with -p (project name)
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "-p",
            "My Project",
            "Auth",
            "Auth module",
        ]);
        assert!(
            args.is_ok(),
            "should parse -p short flag for project name in module add"
        );
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Create { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }

        // Test module ls with --pid alias
        let args = Args::try_parse_from(["dn", "module", "ls", "--pid", "project:abc"]);
        assert!(args.is_ok(), "should parse --pid alias for module list");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Module command");
        }

        // Test module add with --pids alias
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--pids",
            "project:abc",
            "--pids",
            "project:def",
            "Auth",
            "Auth module",
        ]);
        assert!(args.is_ok(), "should parse --pids alias for module add");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Create { project_ids, .. } = command {
                assert_eq!(project_ids.len(), 2);
                assert_eq!(project_ids[0], "project:abc");
                assert_eq!(project_ids[1], "project:def");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn submodule_commands_accepts_short_flags() {
        // Test submodule ls with --pid and --mid aliases
        let args = Args::try_parse_from([
            "dn",
            "submodule",
            "ls",
            "--pid",
            "project:abc",
            "--mid",
            "module:def",
        ]);
        assert!(args.is_ok(), "should parse --pid and --mid aliases");
        if let Commands::Submodule { command } = args.unwrap().command {
            if let SubmoduleCommands::List {
                project_id,
                module_id,
                ..
            } = command
            {
                assert_eq!(project_id, Some("project:abc".to_string()));
                assert_eq!(module_id, Some("module:def".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Submodule command");
        }

        // Test submodule add with --mids alias
        let args = Args::try_parse_from([
            "dn",
            "submodule",
            "add",
            "--mids",
            "module:abc",
            "--mids",
            "module:def",
            "OAuth",
            "OAuth submodule",
        ]);
        assert!(args.is_ok(), "should parse --mids alias");
        if let Commands::Submodule { command } = args.unwrap().command {
            if let SubmoduleCommands::Create { module_ids, .. } = command {
                assert_eq!(module_ids.len(), 2);
                assert_eq!(module_ids[0], "module:abc");
                assert_eq!(module_ids[1], "module:def");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Submodule command");
        }
    }

    #[test]
    fn file_commands_accepts_short_flags() {
        // Test file ls with --pid, --mid, and --smid aliases
        let args = Args::try_parse_from([
            "dn",
            "file",
            "ls",
            "--pid",
            "project:abc",
            "--mid",
            "module:def",
        ]);
        assert!(args.is_ok(), "should parse --pid and --mid aliases");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::List {
                project_id,
                module_id,
                submodule_id,
                ..
            } = command
            {
                assert_eq!(project_id, Some("project:abc".to_string()));
                assert_eq!(module_id, Some("module:def".to_string()));
                assert_eq!(submodule_id, None);
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected File command");
        }

        // Test file ls with --smid alias
        let args = Args::try_parse_from(["dn", "file", "ls", "--smid", "submodule:abc"]);
        assert!(args.is_ok(), "should parse --smid alias");
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
    fn task_commands_accepts_short_flags() {
        // Test task add with -p, --pids, --mids, --usids, and --eids aliases
        let args = Args::try_parse_from([
            "dn",
            "task",
            "add",
            "-p",
            "My Project",
            "--mids",
            "module:abc",
            "--usids",
            "user_story:def",
            "--eids",
            "epic:ghi",
            "Task Name",
            "Task Description",
        ]);
        assert!(args.is_ok(), "should parse task add with all short aliases");
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::Create {
                project,
                module_ids,
                user_story_ids,
                epic_ids,
                ..
            } = command
            {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(module_ids, vec!["module:abc"]);
                assert_eq!(user_story_ids, vec!["user_story:def"]);
                assert_eq!(epic_ids, vec!["epic:ghi"]);
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Task command");
        }

        // Test task ls with --pid alias
        let args = Args::try_parse_from(["dn", "task", "ls", "--pid", "project:abc"]);
        assert!(args.is_ok(), "should parse --pid alias for task list");
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
    fn user_story_commands_accepts_short_flags() {
        // Test user-story add with -p, --pid, and --eids aliases
        let args = Args::try_parse_from([
            "dn",
            "user-story",
            "add",
            "-p",
            "My Project",
            "--eids",
            "epic:abc",
            "As a user",
            "I want to test",
        ]);
        assert!(
            args.is_ok(),
            "should parse user-story add with short aliases"
        );
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::Create {
                project, epic_ids, ..
            } = command
            {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(epic_ids, vec!["epic:abc"]);
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected UserStory command");
        }

        // Test user-story ls with --pid and --eid aliases
        let args = Args::try_parse_from([
            "dn",
            "user-story",
            "ls",
            "--pid",
            "project:abc",
            "--eid",
            "epic:def",
        ]);
        assert!(args.is_ok(), "should parse --pid and --eid aliases");
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::List {
                project_id,
                epic_id,
                ..
            } = command
            {
                assert_eq!(project_id, Some("project:abc".to_string()));
                assert_eq!(epic_id, Some("epic:def".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected UserStory command");
        }
    }

    #[test]
    fn epic_commands_accepts_short_flags() {
        // Test epic add with -p and --pid aliases
        let args = Args::try_parse_from([
            "dn",
            "epic",
            "add",
            "-p",
            "My Project",
            "Epic Title",
            "Epic Description",
        ]);
        assert!(args.is_ok(), "should parse epic add with -p short flag");
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::Create { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Epic command");
        }

        // Test epic ls with --pid alias
        let args = Args::try_parse_from(["dn", "epic", "ls", "--pid", "project:abc"]);
        assert!(args.is_ok(), "should parse --pid alias for epic list");
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Epic command");
        }
    }

    #[test]
    fn todo_commands_accepts_short_flags() {
        // Test todo add with -p and --pids aliases
        let args = Args::try_parse_from(["dn", "todo", "add", "-p", "My Project", "Todo content"]);
        assert!(args.is_ok(), "should parse todo add with -p short flag");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::Create { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Todo command");
        }

        // Test todo add with --pids alias
        let args = Args::try_parse_from([
            "dn",
            "todo",
            "add",
            "--pids",
            "project:abc",
            "--pids",
            "project:def",
            "Todo content",
        ]);
        assert!(args.is_ok(), "should parse --pids alias for todo add");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::Create { project_ids, .. } = command {
                assert_eq!(project_ids.len(), 2);
                assert_eq!(project_ids[0], "project:abc");
                assert_eq!(project_ids[1], "project:def");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Todo command");
        }

        // Test todo ls with --pid alias
        let args = Args::try_parse_from(["dn", "todo", "ls", "--pid", "project:abc"]);
        assert!(args.is_ok(), "should parse --pid alias for todo list");
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::List { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Todo command");
        }
    }

    #[test]
    fn global_flags_accept_short_aliases() {
        // Test --pp alias for --pretty
        let args = Args::try_parse_from(["dn", "--pp", "config", "show"]);
        assert!(args.is_ok(), "should parse --pp alias");
        assert!(args.unwrap().pretty, "pretty should be true");

        // Test --b alias for --backend
        let args = Args::try_parse_from(["dn", "--b", "local", "config", "show"]);
        assert!(args.is_ok(), "should parse --b alias");
        assert_eq!(args.unwrap().backend, Some("local".to_string()));
    }

    #[test]
    fn command_aliases_are_recognized() {
        // Test project aliases
        let args = Args::try_parse_from(["dn", "proj", "ls"]);
        assert!(args.is_ok(), "should parse proj alias");

        let args = Args::try_parse_from(["dn", "prj", "ls"]);
        assert!(args.is_ok(), "should parse prj alias");

        // Test module aliases
        let args = Args::try_parse_from(["dn", "mod", "ls"]);
        assert!(args.is_ok(), "should parse mod alias");

        let args = Args::try_parse_from(["dn", "mdl", "ls"]);
        assert!(args.is_ok(), "should parse mdl alias");

        // Test submodule aliases
        let args = Args::try_parse_from(["dn", "sub", "ls"]);
        assert!(args.is_ok(), "should parse sub alias");

        // Test file aliases
        let args = Args::try_parse_from(["dn", "fi", "ls"]);
        assert!(args.is_ok(), "should parse fi alias");

        // Test task aliases
        let args = Args::try_parse_from(["dn", "tk", "ls"]);
        assert!(args.is_ok(), "should parse tk alias");

        // Test user-story aliases
        let args = Args::try_parse_from(["dn", "us", "ls"]);
        assert!(args.is_ok(), "should parse us alias");

        let args = Args::try_parse_from(["dn", "story", "ls"]);
        assert!(args.is_ok(), "should parse story alias");

        // Test epic aliases
        let args = Args::try_parse_from(["dn", "ep", "ls"]);
        assert!(args.is_ok(), "should parse ep alias");

        // Test todo aliases
        let args = Args::try_parse_from(["dn", "td", "ls"]);
        assert!(args.is_ok(), "should parse td alias");

        // Test config aliases
        let args = Args::try_parse_from(["dn", "cfg", "show"]);
        assert!(args.is_ok(), "should parse cfg alias");

        // Test link alias
        let args = Args::try_parse_from([
            "dn",
            "ln",
            "-f",
            "project:abc",
            "-e",
            "contains",
            "-t",
            "module:def",
        ]);
        assert!(args.is_ok(), "should parse ln alias");
    }

    #[test]
    fn mixed_short_and_long_flags_work_together() {
        // Test mixing short and long flags in add command
        let args = Args::try_parse_from([
            "dn",
            "add",
            "-f",
            "type",
            "--value",
            "mistake",
            "-f",
            "content",
            "-v",
            "Mixed flags test",
            "--ln",
            "project:abc",
        ]);
        assert!(args.is_ok(), "should parse mixed short and long flags");
        if let Commands::Add {
            field_names,
            field_values,
            link_to,
            ..
        } = args.unwrap().command
        {
            assert_eq!(field_names.len(), 2);
            assert_eq!(field_values.len(), 2);
            assert_eq!(link_to.len(), 1);
        } else {
            panic!("expected Add command");
        }
    }
}
