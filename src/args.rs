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
        long_about = "Persist one knowledge entry with category metadata so it can be retrieved as coding context.",
        after_help = "Examples:\n  lazydev add --category rust --type mistake --content \"Avoid unwrap in library code\"\n  lazydev add -c backend -t skill -C \"Design resilient APIs\""
    )]
    Add {
        /// Knowledge category (for graph/tag seeding).
        #[arg(short, long, value_name = "CATEGORY")]
        category: String,

        /// Knowledge type (`mistake`, `style`, or `skill`).
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        kind: String,

        /// Main content to store.
        #[arg(short = 'C', long, value_name = "CONTENT")]
        content: String,
    },
    #[command(
        about = "Retrieve coding context for a query.",
        long_about = "Find relevant mistakes, style rules, and skills by deriving graph seeds from the query and traversing related nodes.",
        after_help = "Example:\n  lazydev context \"how to handle rust errors without unwrap\""
    )]
    Context {
        /// Natural-language query to retrieve context for.
        #[arg(value_name = "QUERY")]
        query: String,
    },
}
