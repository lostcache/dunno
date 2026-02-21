use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lazydev",
    author,
    version,
    about = "Capture and retrieve coding knowledge from mistakes, style guides, and security details.",
    long_about = "lazydev stores coding knowledge in a graph database and retrieves context via deterministic hierarchy traversal.",
    propagate_version = true
)]
pub struct Args {
    /// Optional storage backend override (`local` or `cloud`).
    #[arg(long, global = true, value_name = "BACKEND")]
    pub backend: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Add a new knowledge entry.",
        long_about = "Persist one knowledge entry (mistake/style/security) and optionally link it to a structural node.",
        after_help = "Examples:\n  lazydev add --type mistake --content \"Avoid unwrap\"\n  lazydev add --type security --content \"SQL injection risk\" --link-to module:abc"
    )]
    Add {
        /// Knowledge type (`mistake`, `style`, or `security`).
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        kind: String,

        /// Main content to store.
        #[arg(short = 'C', long, value_name = "CONTENT")]
        content: String,

        /// Optional ID of a structural node to link this knowledge to.
        #[arg(long, value_name = "LINK_TO")]
        link_to: Option<String>,
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
        long_about = "Find relevant context by traversing the structural hierarchy via graph edges.",
        after_help = "Example:\n  lazydev context --task-id task:123\n  lazydev context --file-id file:456\n  lazydev context --subtask-id subtask:789"
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
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    Create { name: String, description: String },
    List,
}

#[derive(Subcommand, Debug)]
pub enum ModuleCommands {
    Create {
        #[arg(long)]
        project_id: String,
        name: String,
        description: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
pub enum SubmoduleCommands {
    Create {
        #[arg(long)]
        module_id: String,
        name: String,
        description: String,
    },
    List {
        #[arg(long)]
        module_id: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum FileCommands {
    Create {
        /// Parent ID (module or submodule).
        #[arg(long)]
        parent_id: String,
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

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    Create {
        #[arg(long)]
        module_id: String,
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
    AppendUpdate {
        task_id: String,
        content: String,
    },
    UpdateEntry {
        update_id: String,
        content: String,
    },
    ListUpdates {
        task_id: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
pub enum SubtaskCommands {
    Create {
        #[arg(long)]
        task_id: String,
        name: String,
        description: String,
    },
    List {
        #[arg(long)]
        task_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    Create {
        #[arg(long)]
        project_id: String,
        content: String,
    },
    List {
        #[arg(long)]
        project_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    Show,
}
