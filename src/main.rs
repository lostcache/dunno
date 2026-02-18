use clap::Parser;
use lazydev::args::{Args, Commands};
use lazydev::config::Config;
use lazydev::context::get_context;
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::vector_db::VectorDB;
use serde_json::json;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(err) = run(args).await {
        println!("{}", json!({ "error": err.to_string() }));
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::default();
    let db = DB::new(&config.surreal_url).await?;
    let vector_db = match VectorDB::new(&config.qdrant_url).await {
        Ok(db) => db,
        Err(_) => VectorDB::new("mem://").await?,
    };

    match args.command {
        Commands::Add {
            category,
            kind,
            content,
        } => {
            add_knowledge(category, kind, content, &db, &vector_db).await?;
            println!("{}", json!({ "status": "ok" }));
        }
        Commands::Context { query } => {
            let results = get_context(query, &db, &vector_db).await?;
            println!("{}", json!({ "results": results }));
        }
    }

    Ok(())
}
