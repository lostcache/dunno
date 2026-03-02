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
        long_about = "Persist one knowledge entry (mistake/style/security) and optionally link it to a structural node.",
        after_help = "Examples:\n  dunno add --type mistake --content \"Avoid unwrap\"\n  dunno add --type security --content \"SQL injection risk\" --link-to module:abc"
    )]
    Add {
        /// Knowledge type (`mistake`, `style`, or `security`).
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        kind: String,

        /// Main content to store.
        #[arg(short = 'C', long, value_name = "CONTENT")]
        content: String,

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
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["file_id", "subtask_id"])]
        task_id: Option<String>,

        /// The File ID to retrieve context for.
        #[arg(long, value_name = "FILE_ID", conflicts_with_all = ["task_id", "subtask_id"])]
        file_id: Option<String>,

        /// The Subtask ID to retrieve context for.
        #[arg(long, value_name = "SUBTASK_ID", conflicts_with_all = ["task_id", "file_id"])]
        subtask_id: Option<String>,
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
