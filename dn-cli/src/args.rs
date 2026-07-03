use crate::epic::EpicCommands;
use crate::file::FileCommands;
use crate::issue::IssueCommands;
use crate::module::ModuleCommands;
use crate::persona::PersonaCommands;
use crate::project::ProjectCommands;
use crate::task::TaskCommands;
use crate::todo::TodoCommands;
use crate::user_story::UserStoryCommands;
use crate::workflow::WorkflowCommands;

#[derive(clap::Parser, Debug)]
#[command(
    name = "dn",
    author,
    version,
    about = "Just a couple of binaries to replace all your (except one) md files",
    propagate_version = true
)]
pub struct Args {
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
        about = "Add a new knowledge entry and optionally link it to other nodes.",
        after_help = "Examples:\
            \n  dn add --field type --value mistake --field content --value \"Avoid unwrap\" --field severity --value high\
            \n  dn add --field type --value security --field content --value \"SQL injection risk\" --link-to module:abc\
            \n  dn add --field custom_type --value performance --field content --value \"Use parallel iterators\" --field category --value optimization"
    )]
    Add {
        #[arg(
            short = 'f',
            long = "field",
            value_name = "FIELD_NAME",
            required = true
        )]
        field_names: Vec<String>,
        #[arg(
            short = 'v',
            long = "value",
            value_name = "FIELD_VALUE",
            required = true
        )]
        field_values: Vec<String>,
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

    #[command(about = "Manage personas.", visible_alias = "per")]
    Persona {
        #[command(subcommand)]
        command: PersonaCommands,
    },

    #[command(about = "Manage workflows.", visible_alias = "wf")]
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
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

    #[command(about = "Manage issues.", visible_alias = "iss")]
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
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
        about = "Retrieve context for a task, file, epic, or project.\n\
            \n\
            Traverses the knowledge graph and returns linked context.\n\
            \n\
            - Use --general -p <PROJECT> for the full project structure.\n\
            - Use --full to retrieve all linked nodes.\n\
            - Use --task-id, --file-id, or --epic-id for entity-scoped context.",
        after_help = "Example:\
            \n  dn ctx --task-id task:123\
            \n  dn ctx --task-id task:123 --full\
            \n  dn ctx --file-id file:456\
            \n  dn ctx --epic-id epic:789\
            \n  dn ctx --general -p MyProject"
    )]
    Context {
        #[arg(long, visible_alias = "tid", value_name = "TASK_ID", help = "Fetch context for this task (inherits from module and project).", conflicts_with_all = ["file_id", "epic_id", "general", "project"])]
        task_id: Option<String>,
        #[arg(long, visible_alias = "fid", value_name = "FILE_ID", help = "Fetch context for this file (inherits from module and project).", conflicts_with_all = ["task_id", "epic_id", "general", "project"])]
        file_id: Option<String>,
        #[arg(long, visible_alias = "eid", value_name = "EPIC_ID", help = "Fetch context for this epic.", conflicts_with_all = ["task_id", "file_id", "general", "project"])]
        epic_id: Option<String>,
        #[arg(
            long,
            help = "Include the full inherited hierarchy (modules, project) not just direct context."
        )]
        full: bool,
        #[arg(long, help = "Return the full project structure (modules, files, tasks). Requires --project.", conflicts_with_all = ["task_id", "file_id", "epic_id"])]
        general: bool,
        #[arg(
            short = 'p',
            long,
            value_name = "PROJECT",
            help = "Project name or ID. Required with --general.",
            requires = "general"
        )]
        project: Option<String>,
    },

    #[command(
        about = "Link a source node to one or more target nodes via a named edge.",
        visible_alias = "ln",
        after_help = "Example:\
            \n  dn link --from-id project:abc --edge contains --to-id module:def\
            \n  dn link --from-id project:abc --edge has_todo --to-id todo_item:1 --to-id todo_item:2\
            \n  dn link --from-id file:A --edge belongs_to_task --to-id task:X --from-id file:B --edge belongs_to_task --to-id task:X"
    )]
    Link {
        #[arg(short, long, value_name = "FROM_ID")]
        from_id: Vec<String>,
        #[arg(short, long, value_name = "EDGE")]
        edge: Vec<String>,
        #[arg(short, long, value_name = "TO_ID")]
        to_id: Vec<String>,
    },

    #[command(
        about = "Delete one or more context entries by ID.",
        after_help = "Example:\
            \n  dn rm context:abc\
            \n  dn rm context:abc context:def"
    )]
    Rm {
        #[arg(required = true, value_name = "CONTEXT_ID")]
        context_ids: Vec<String>,
    },

    #[command(
        about = "Delete all records from the database. This action is irreversible. (DANGER)",
        hide = true
    )]
    Purge,
}

#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommands {
    Show,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epic::EpicCommands;
    use crate::file::FileCommands;
    use crate::issue::IssueCommands;
    use crate::module::ModuleCommands;
    use crate::task::TaskCommands;
    use crate::todo::TodoCommands;
    use crate::user_story::UserStoryCommands;
    use clap::Parser;

    #[test]
    fn context_command_enforces_mutual_exclusion_of_ids() {
        let result =
            Args::try_parse_from(["dn", "ctx", "--task-id", "task:1", "--file-id", "file:2"]);
        assert!(result.is_err(), "expected clap to reject conflicting ids");

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
        let result = Args::try_parse_from(["dn", "add", "--field", "type"]);
        assert!(
            result.is_err(),
            "expected clap to require --value when --field is present"
        );

        let result2 = Args::try_parse_from(["dn", "add", "--value", "mistake"]);
        assert!(
            result2.is_err(),
            "expected clap to require --field when --value is present"
        );

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
            if let TaskCommands::Delete { task_ids } = command {
                assert_eq!(task_ids, vec!["task:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Task command");
        }
    }

    #[test]
    fn task_delete_command_accepts_multiple_ids() {
        let args = Args::try_parse_from(["dn", "task", "rm", "task:abc123", "task:def456"]);
        assert!(
            args.is_ok(),
            "should parse task delete command with multiple ids"
        );
        if let Commands::Task { command } = args.unwrap().command {
            if let TaskCommands::Delete { task_ids } = command {
                assert_eq!(task_ids, vec!["task:abc123", "task:def456"]);
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
            if let TodoCommands::Delete { todo_ids } = command {
                assert_eq!(todo_ids, vec!["todo_item:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Todo command");
        }
    }

    #[test]
    fn todo_delete_command_accepts_multiple_ids() {
        let args =
            Args::try_parse_from(["dn", "todo", "rm", "todo_item:abc123", "todo_item:def456"]);
        assert!(
            args.is_ok(),
            "should parse todo delete command with multiple ids"
        );
        if let Commands::Todo { command } = args.unwrap().command {
            if let TodoCommands::Delete { todo_ids } = command {
                assert_eq!(todo_ids, vec!["todo_item:abc123", "todo_item:def456"]);
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
        let args = Args::try_parse_from([
            "dn",
            "--pretty",
            "file",
            "add",
            "--parent-mod-id",
            "module:abc",
            "--name",
            "main.rs",
            "--path",
            "src/main.rs",
        ]);
        assert!(args.is_ok(), "should parse --pretty with file create");
        assert!(args.unwrap().pretty, "pretty should be true");

        let args_with_desc = Args::try_parse_from([
            "dn",
            "--pretty",
            "file",
            "add",
            "--parent-mod-id",
            "module:abc",
            "--name",
            "main.rs",
            "--path",
            "src/main.rs",
            "--description",
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
            "--to-id",
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
            "--name",
            "Auth",
            "--desc",
            "Auth module",
        ]);
        assert!(args.is_ok(), "should parse --project with module create");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Add { project, name, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(name[0], "Auth");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_create_rejects_both_project_and_project_id() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "--project-id",
            "project:abc",
            "Auth",
            "Auth module",
        ]);
        assert!(
            args.is_err(),
            "should reject both --project and --project-id"
        );
    }

    #[test]
    fn module_add_multiple_modules() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "--name",
            "Auth",
            "--desc",
            "Auth module",
            "--name",
            "Auth2",
            "--desc",
            "Auth2 module",
        ]);
        assert!(args.is_ok(), "should parse --project with module create");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Add { project, name, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(name[0], "Auth");
                assert_eq!(name[1], "Auth2");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn add_multiple_modules_with_parent_module() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "--name",
            "Auth",
            "--desc",
            "Auth module",
            "--name",
            "Auth2",
            "--desc",
            "Auth2 module",
            "--pmid",
            "module:abc",
            "--pmid",
            "module:abc",
        ]);
        assert!(args.is_ok(), "should parse --project with module create");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Add {
                project,
                name,
                parent_module_id,
                ..
            } = command
            {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(name[0], "Auth");
                assert_eq!(name[1], "Auth2");
                assert_eq!(parent_module_id[0], "module:abc");
                assert_eq!(parent_module_id[1], "module:abc");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_add_multiple_modules_mixed_parents() {
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "--project",
            "My Project",
            "--name",
            "Top",
            "--desc",
            "top",
            "--name",
            "Child",
            "--desc",
            "child",
            "--pmid",
            "",
            "--pmid",
            "module:abc",
        ]);
        assert!(args.is_ok(), "should parse mixed top-level and child");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Add {
                name,
                parent_module_id,
                ..
            } = command
            {
                assert_eq!(name.len(), 2);
                assert_eq!(parent_module_id.len(), 2);
                assert_eq!(parent_module_id[0], "");
                assert_eq!(parent_module_id[1], "module:abc");
            } else {
                panic!("expected Add command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn file_add_multiple_files() {
        let args = Args::try_parse_from([
            "dn",
            "file",
            "add",
            "--project",
            "My Project",
            "--name",
            "a.rs",
            "--path",
            "src/a.rs",
            "--description",
            "first",
            "--parent-mod-id",
            "module:abc",
            "--name",
            "b.rs",
            "--path",
            "src/b.rs",
            "--description",
            "second",
            "--parent-mod-id",
            "module:def",
        ]);
        assert!(args.is_ok(), "should parse multiple files in one add");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::Add {
                name,
                path,
                description,
                parent_mod_id,
                ..
            } = command
            {
                assert_eq!(name.len(), 2);
                assert_eq!(path.len(), 2);
                assert_eq!(description.len(), 2);
                assert_eq!(parent_mod_id.len(), 2);
                assert_eq!(name[0], "a.rs");
                assert_eq!(path[0], "src/a.rs");
                assert_eq!(description[0], "first");
                assert_eq!(parent_mod_id[0], "module:abc");
                assert_eq!(name[1], "b.rs");
                assert_eq!(path[1], "src/b.rs");
                assert_eq!(description[1], "second");
                assert_eq!(parent_mod_id[1], "module:def");
            } else {
                panic!("expected Add command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn file_add_mixed_parent_and_freestanding() {
        let args = Args::try_parse_from([
            "dn",
            "file",
            "add",
            "--project",
            "My Project",
            "--name",
            "linked.rs",
            "--path",
            "src/linked.rs",
            "--description",
            "d1",
            "--parent-mod-id",
            "module:abc",
            "--name",
            "free.rs",
            "--path",
            "src/free.rs",
            "--description",
            "d2",
            "--parent-mod-id",
            "",
        ]);
        assert!(args.is_ok(), "should parse linked + freestanding files");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::Add { parent_mod_id, .. } = command {
                assert_eq!(parent_mod_id.len(), 2);
                assert_eq!(parent_mod_id[0], "module:abc");
                assert_eq!(parent_mod_id[1], "");
            } else {
                panic!("expected Add command");
            }
        } else {
            panic!("expected File command");
        }
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
    fn task_create_requires_project() {
        let args = Args::try_parse_from(["dn", "task", "add", "Task Name", "Description"]);
        assert!(
            args.is_err(),
            "task add without a project should be rejected"
        );

        let args = Args::try_parse_from([
            "dn",
            "task",
            "add",
            "--project-id",
            "project:abc",
            "Task Name",
            "Description",
        ]);
        assert!(
            args.is_ok(),
            "task add with only --project-id should be accepted"
        );
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
            if let ProjectCommands::Delete { project_ids } = command {
                assert_eq!(project_ids, vec!["project:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Project command");
        }
    }

    #[test]
    fn project_delete_command_accepts_multiple_ids() {
        let args =
            Args::try_parse_from(["dn", "project", "rm", "project:abc123", "project:def456"]);
        assert!(
            args.is_ok(),
            "should parse project delete command with multiple ids"
        );
        if let Commands::Project { command } = args.unwrap().command {
            if let ProjectCommands::Delete { project_ids } = command {
                assert_eq!(project_ids, vec!["project:abc123", "project:def456"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Project command");
        }
    }

    #[test]
    fn project_delete_command_requires_project_id() {
        let args = Args::try_parse_from(["dn", "project", "rm"]);
        assert!(
            args.is_err(),
            "should require project_id for delete command"
        );
    }

    #[test]
    fn module_delete_command_accepts_module_id() {
        let args = Args::try_parse_from(["dn", "module", "rm", "module:abc123"]);
        assert!(args.is_ok(), "should parse module delete command");
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Delete { module_ids } = command {
                assert_eq!(module_ids, vec!["module:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_delete_command_accepts_multiple_ids() {
        let args = Args::try_parse_from(["dn", "module", "rm", "module:abc123", "module:def456"]);
        assert!(
            args.is_ok(),
            "should parse module delete command with multiple ids"
        );
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Delete { module_ids } = command {
                assert_eq!(module_ids, vec!["module:abc123", "module:def456"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Module command");
        }
    }

    #[test]
    fn module_delete_command_requires_module_id() {
        let args = Args::try_parse_from(["dn", "module", "rm"]);
        assert!(args.is_err(), "should require module_id for delete command");
    }

    #[test]
    fn file_delete_command_accepts_file_id() {
        let args = Args::try_parse_from(["dn", "file", "rm", "file:abc123"]);
        assert!(args.is_ok(), "should parse file delete command");
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::Delete { file_ids } = command {
                assert_eq!(file_ids, vec!["file:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn file_delete_command_accepts_multiple_ids() {
        let args = Args::try_parse_from(["dn", "file", "rm", "file:abc123", "file:def456"]);
        assert!(
            args.is_ok(),
            "should parse file delete command with multiple ids"
        );
        if let Commands::File { command } = args.unwrap().command {
            if let FileCommands::Delete { file_ids } = command {
                assert_eq!(file_ids, vec!["file:abc123", "file:def456"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn file_delete_command_requires_file_id() {
        let args = Args::try_parse_from(["dn", "file", "rm"]);
        assert!(args.is_err(), "should require file_id for delete command");
    }

    #[test]
    fn user_story_delete_command_accepts_user_story_id() {
        let args = Args::try_parse_from(["dn", "user-story", "rm", "user_story:abc123"]);
        assert!(args.is_ok(), "should parse user-story delete command");
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::Delete { user_story_ids } = command {
                assert_eq!(user_story_ids, vec!["user_story:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected UserStory command");
        }
    }

    #[test]
    fn user_story_delete_command_accepts_multiple_ids() {
        let args = Args::try_parse_from([
            "dn",
            "user-story",
            "rm",
            "user_story:abc123",
            "user_story:def456",
        ]);
        assert!(
            args.is_ok(),
            "should parse user-story delete command with multiple ids"
        );
        if let Commands::UserStory { command } = args.unwrap().command {
            if let UserStoryCommands::Delete { user_story_ids } = command {
                assert_eq!(
                    user_story_ids,
                    vec!["user_story:abc123", "user_story:def456"]
                );
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected UserStory command");
        }
    }

    #[test]
    fn user_story_delete_command_requires_user_story_id() {
        let args = Args::try_parse_from(["dn", "user-story", "rm"]);
        assert!(
            args.is_err(),
            "should require user_story_id for delete command"
        );
    }

    #[test]
    fn epic_delete_command_accepts_epic_id() {
        let args = Args::try_parse_from(["dn", "epic", "rm", "epic:abc123"]);
        assert!(args.is_ok(), "should parse epic delete command");
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::Delete { epic_ids } = command {
                assert_eq!(epic_ids, vec!["epic:abc123"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Epic command");
        }
    }

    #[test]
    fn epic_delete_command_accepts_multiple_ids() {
        let args = Args::try_parse_from(["dn", "epic", "rm", "epic:abc123", "epic:def456"]);
        assert!(
            args.is_ok(),
            "should parse epic delete command with multiple ids"
        );
        if let Commands::Epic { command } = args.unwrap().command {
            if let EpicCommands::Delete { epic_ids } = command {
                assert_eq!(epic_ids, vec!["epic:abc123", "epic:def456"]);
            } else {
                panic!("expected Delete command");
            }
        } else {
            panic!("expected Epic command");
        }
    }

    #[test]
    fn epic_delete_command_requires_epic_id() {
        let args = Args::try_parse_from(["dn", "epic", "rm"]);
        assert!(args.is_err(), "should require epic_id for delete command");
    }

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
            to_id,
        } = args.unwrap().command
        {
            assert_eq!(from_id.len(), 1);
            assert_eq!(from_id[0], "project:abc");
            assert_eq!(edge.len(), 1);
            assert_eq!(edge[0], "contains");
            assert_eq!(to_id.len(), 2);
            assert_eq!(to_id[0], "module:def");
            assert_eq!(to_id[1], "module:ghi");
        } else {
            panic!("expected Link command");
        }
    }

    #[test]
    fn link_command_accepts_multiple_triplets() {
        let args = Args::try_parse_from([
            "dn",
            "link",
            "--from-id",
            "file:A",
            "--edge",
            "belongs_to_task",
            "--to-id",
            "task:X",
            "--from-id",
            "file:B",
            "--edge",
            "belongs_to_task",
            "--to-id",
            "task:X",
        ]);
        assert!(args.is_ok(), "should parse multiple triplets for link");
        if let Commands::Link {
            from_id,
            edge,
            to_id,
        } = args.unwrap().command
        {
            assert_eq!(from_id.len(), 2);
            assert_eq!(from_id[0], "file:A");
            assert_eq!(from_id[1], "file:B");
            assert_eq!(edge.len(), 2);
            assert_eq!(edge[0], "belongs_to_task");
            assert_eq!(edge[1], "belongs_to_task");
            assert_eq!(to_id.len(), 2);
            assert_eq!(to_id[0], "task:X");
            assert_eq!(to_id[1], "task:X");
        } else {
            panic!("expected Link command");
        }
    }

    #[test]
    fn context_command_accepts_short_id_flags() {
        let args = Args::try_parse_from(["dn", "ctx", "--tid", "task:abc"]);
        assert!(args.is_ok(), "should parse --tid alias");
        if let Commands::Context { task_id, .. } = args.unwrap().command {
            assert_eq!(task_id, Some("task:abc".to_string()));
        } else {
            panic!("expected Context command");
        }

        let args = Args::try_parse_from(["dn", "ctx", "--fid", "file:def"]);
        assert!(args.is_ok(), "should parse --fid alias");
        if let Commands::Context { file_id, .. } = args.unwrap().command {
            assert_eq!(file_id, Some("file:def".to_string()));
        } else {
            panic!("expected Context command");
        }

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
        let args = Args::try_parse_from([
            "dn",
            "module",
            "add",
            "-p",
            "My Project",
            "--name",
            "Auth",
            "--desc",
            "Auth module",
        ]);
        assert!(
            args.is_ok(),
            "should parse -p short flag for project name in module add"
        );
        if let Commands::Module { command } = args.unwrap().command {
            if let ModuleCommands::Add { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Module command");
        }

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
    }

    #[test]
    fn file_commands_accepts_short_flags() {
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
                ..
            } = command
            {
                assert_eq!(project_id, Some("project:abc".to_string()));
                assert_eq!(module_id, Some("module:def".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected File command");
        }
    }

    #[test]
    fn task_commands_accepts_short_flags() {
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
        let args = Args::try_parse_from(["dn", "--pp", "config", "show"]);
        assert!(args.is_ok(), "should parse --pp alias");
        assert!(args.unwrap().pretty, "pretty should be true");
    }

    #[test]
    fn command_aliases_are_recognized() {
        let args = Args::try_parse_from(["dn", "proj", "ls"]);
        assert!(args.is_ok(), "should parse proj alias");

        let args = Args::try_parse_from(["dn", "prj", "ls"]);
        assert!(args.is_ok(), "should parse prj alias");

        let args = Args::try_parse_from(["dn", "mod", "ls"]);
        assert!(args.is_ok(), "should parse mod alias");

        let args = Args::try_parse_from(["dn", "mdl", "ls"]);
        assert!(args.is_ok(), "should parse mdl alias");

        let args = Args::try_parse_from(["dn", "fi", "ls"]);
        assert!(args.is_ok(), "should parse fi alias");

        let args = Args::try_parse_from(["dn", "tk", "ls"]);
        assert!(args.is_ok(), "should parse tk alias");

        let args = Args::try_parse_from(["dn", "us", "ls"]);
        assert!(args.is_ok(), "should parse us alias");

        let args = Args::try_parse_from(["dn", "story", "ls"]);
        assert!(args.is_ok(), "should parse story alias");

        let args = Args::try_parse_from(["dn", "ep", "ls"]);
        assert!(args.is_ok(), "should parse ep alias");

        let args = Args::try_parse_from(["dn", "td", "ls"]);
        assert!(args.is_ok(), "should parse td alias");

        let args = Args::try_parse_from(["dn", "cfg", "show"]);
        assert!(args.is_ok(), "should parse cfg alias");

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

    #[test]
    fn issue_add_parses_description() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project-id",
            "project:abc",
            "Users cannot log in",
        ]);
        assert!(args.is_ok(), "should parse issue add");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create {
                task_id,
                description,
                ..
            } = command
            {
                assert_eq!(task_id, None);
                assert_eq!(description, "Users cannot log in");
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_task_id() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project-id",
            "project:abc",
            "--task-id",
            "task:abc123",
            "Desc",
        ]);
        assert!(args.is_ok(), "should parse issue add with --task-id");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create { task_id, .. } = command {
                assert_eq!(task_id, Some("task:abc123".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_tid_alias() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project-id",
            "project:abc",
            "--tid",
            "task:abc123",
            "Desc",
        ]);
        assert!(args.is_ok(), "should parse issue add with --tid alias");
    }

    #[test]
    fn issue_list_requires_project_id() {
        let args = Args::try_parse_from(["dn", "issue", "ls"]);
        assert!(args.is_err(), "should require --project-id");
    }

    #[test]
    fn issue_list_accepts_project_id() {
        let args = Args::try_parse_from(["dn", "issue", "ls", "--project-id", "project:abc"]);
        assert!(args.is_ok(), "should parse issue list with --project-id");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::List {
                project_id,
                task_id,
            } = command
            {
                assert_eq!(project_id, "project:abc".to_string());
                assert_eq!(task_id, None);
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_list_accepts_task_id() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "ls",
            "--project-id",
            "project:abc",
            "--task-id",
            "task:abc",
        ]);
        assert!(args.is_ok(), "should parse issue list with --task-id");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::List { task_id, .. } = command {
                assert_eq!(task_id, Some("task:abc".to_string()));
            } else {
                panic!("expected List command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_project_id() {
        let args =
            Args::try_parse_from(["dn", "issue", "add", "--project-id", "project:abc", "Desc"]);
        assert!(args.is_ok(), "should parse issue add with --project-id");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create { project_id, .. } = command {
                assert_eq!(project_id, Some("project:abc".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_project_name() {
        let args = Args::try_parse_from(["dn", "issue", "add", "--project", "My Project", "Desc"]);
        assert!(args.is_ok(), "should parse issue add with --project");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create {
                project,
                description,
                ..
            } = command
            {
                assert_eq!(project, Some("My Project".to_string()));
                assert_eq!(description, "Desc".to_string());
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_project_name_short_flag() {
        let args = Args::try_parse_from(["dn", "issue", "add", "-p", "My Project", "Desc"]);
        assert!(args.is_ok(), "should parse issue add with -p");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create { project, .. } = command {
                assert_eq!(project, Some("My Project".to_string()));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_rejects_both_project_and_project_id() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project",
            "My Project",
            "--project-id",
            "project:abc",
            "Desc",
        ]);
        assert!(
            args.is_err(),
            "should reject both --project and --project-id"
        );
    }

    #[test]
    fn issue_rm_requires_at_least_one_id() {
        let args = Args::try_parse_from(["dn", "issue", "rm"]);
        assert!(args.is_err(), "should reject issue rm with no ids");
    }

    #[test]
    fn issue_rm_accepts_single_id() {
        let args = Args::try_parse_from(["dn", "issue", "rm", "issue:abc123"]);
        assert!(args.is_ok(), "should parse issue rm with one id");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Remove { issue_ids } = command {
                assert_eq!(issue_ids, vec!["issue:abc123"]);
            } else {
                panic!("expected Remove command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_rm_accepts_multiple_ids() {
        let args = Args::try_parse_from(["dn", "issue", "rm", "issue:abc", "issue:def"]);
        assert!(args.is_ok(), "should parse issue rm with multiple ids");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Remove { issue_ids } = command {
                assert_eq!(issue_ids, vec!["issue:abc", "issue:def"]);
            } else {
                panic!("expected Remove command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_update_parses_all_fields() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "update",
            "issue:abc123",
            "--description",
            "New Desc",
            "--plan",
            "Fix it",
            "--status",
            "active",
        ]);
        assert!(args.is_ok(), "should parse issue update");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Update {
                issue_id,
                description,
                plan,
                status,
                ..
            } = command
            {
                assert_eq!(issue_id, "issue:abc123");
                assert_eq!(description.as_deref(), Some("New Desc"));
                assert_eq!(plan.as_deref(), Some("Fix it"));
                assert_eq!(status.as_deref(), Some("active"));
            } else {
                panic!("expected Update command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_accepts_verification() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project-id",
            "project:abc",
            "--verification",
            "Check the login flow",
            "Users cannot log in",
        ]);
        assert!(args.is_ok(), "should parse issue add with --verification");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create { verification, .. } = command {
                assert_eq!(verification.as_deref(), Some("Check the login flow"));
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_add_verification_defaults_to_none() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "add",
            "--project-id",
            "project:abc",
            "Users cannot log in",
        ]);
        assert!(args.is_ok());
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Create { verification, .. } = command {
                assert!(verification.is_none());
            } else {
                panic!("expected Create command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_update_accepts_verification() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "update",
            "issue:abc123",
            "--verification",
            "Verify fix in staging",
        ]);
        assert!(
            args.is_ok(),
            "should parse issue update with --verification"
        );
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Update { verification, .. } = command {
                assert_eq!(verification.as_deref(), Some("Verify fix in staging"));
            } else {
                panic!("expected Update command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_update_verification_defaults_to_none() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "update",
            "issue:abc123",
            "--status",
            "active",
        ]);
        assert!(args.is_ok());
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Update { verification, .. } = command {
                assert!(verification.is_none());
            } else {
                panic!("expected Update command");
            }
        } else {
            panic!("expected Issue command");
        }
    }

    #[test]
    fn issue_update_requires_issue_id() {
        let args = Args::try_parse_from(["dn", "issue", "update"]);
        assert!(args.is_err(), "should require issue_id");
    }

    #[test]
    fn issue_update_accepts_partial_fields() {
        let args = Args::try_parse_from([
            "dn",
            "issue",
            "update",
            "issue:abc123",
            "--status",
            "completed",
        ]);
        assert!(args.is_ok(), "should parse issue update with only status");
        if let Commands::Issue { command } = args.unwrap().command {
            if let IssueCommands::Update {
                issue_id,
                description,
                status,
                ..
            } = command
            {
                assert_eq!(issue_id, "issue:abc123");
                assert!(description.is_none());
                assert_eq!(status.as_deref(), Some("completed"));
            } else {
                panic!("expected Update command");
            }
        } else {
            panic!("expected Issue command");
        }
    }
}
