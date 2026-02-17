use clap::Parser;
use lazydev::args::{Args, Commands};

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Add {
            category,
            kind,
            content,
        } => {
            println!(
                "Adding knowledge: Category={}, Type={}, Content={}",
                category, kind, content
            );
        }
        Commands::Context { query } => {
            println!("Searching context for: {}", query);
        }
    }
}
