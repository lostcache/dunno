use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lazydev",
    author,
    version,
    about = "Capture and retrieve coding knowledge from mistakes, style guides, and skills.",
    long_about = "lazydev stores coding knowledge in a local graph+vector setup and retrieves context for natural-language queries.",
    propagate_version = true
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Add a new knowledge entry.",
        long_about = "Persist one knowledge entry (Mistake/Style/Skill) so it can be linked to tasks.",
        after_help = "Examples:\n  lazydev add --category rust --type mistake --content \"Avoid unwrap\""
    )]
    Add {
        /// Knowledge category (for tagging).
        #[arg(short, long, value_name = "CATEGORY")]
        category: String,

        /// Knowledge type (`mistake`, `style`, or `skill`).
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        kind: String,

        /// Main content to store.
        #[arg(short = 'C', long, value_name = "CONTENT")]
        content: String,

        /// Optional ID of a Project/Module/Task to link this knowledge to.
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

    #[command(about = "Manage tasks.")]
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    #[command(about = "Manage todo items.")]
    Todo {
        #[command(subcommand)]
        command: TodoCommands,
    },

    #[command(
        about = "Retrieve coding context for a task.",
        long_about = "Find relevant context by traversing the Project -> Module -> Task hierarchy.",
        after_help = "Example:\n  lazydev context --task-id task:123"
    )]
    Context {
        /// The Task ID to retrieve context for.
        #[arg(long, value_name = "TASK_ID")]
        task_id: String,
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
        project_id: String,
        name: String,
        description: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    Create {
        module_id: String,
        name: String,
        description: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    Create { project_id: String, content: String },
    List { project_id: String },
}
