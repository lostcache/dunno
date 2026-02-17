use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add new knowledge
    Add {
        /// Category of the knowledge
        #[arg(short, long)]
        category: String,

        /// Type of the knowledge (e.g., mistake, style, skill)
        #[arg(short = 't', long = "type")]
        kind: String,

        /// Content of the knowledge
        #[arg(short = 'C', long)]
        content: String,
    },
    /// Get context based on query
    Context {
        /// The query string
        query: String,
    },
}
