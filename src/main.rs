use clap::error::ErrorKind;
use clap::Parser;
use lazydev::args::{Args, Commands, ModuleCommands, ProjectCommands, TaskCommands, TodoCommands};
use lazydev::config::Config;
use lazydev::context::get_task_context;
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::models::{Module, Project, Task, TodoItem};
use lazydev::vector_db::VectorDB;
use serde_json::json;

#[tokio::main]
async fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            if matches!(err.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                print!("{}", err);
                return;
            }
            print_error_json("cli_parse_error", err.to_string());
            std::process::exit(2);
        }
    };

    if let Err(err) = run(args).await {
        print_error_json("runtime_error", err.to_string());
        std::process::exit(1);
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
            link_to,
        } => {
            add_knowledge(category, kind, content, link_to, &db, &vector_db).await?;
            println!("{}", json!({ "status": "ok" }));
        }
        Commands::Project { command } => match command {
            ProjectCommands::Create { name, description } => {
                let project = Project {
                    id: None,
                    name,
                    description,
                };
                let created = db.create_project(&project).await?;
                println!("{}", json!(created));
            }
            ProjectCommands::List => {
                let projects = db.list_projects().await?;
                println!("{}", json!(projects));
            }
        },
        Commands::Module { command } => match command {
            ModuleCommands::Create {
                project_id,
                name,
                description,
            } => {
                let module = Module {
                    id: None,
                    project_id,
                    name,
                    description,
                };
                let created = db.create_module(&module).await?;
                println!("{}", json!(created));
            }
            ModuleCommands::List => {
                let modules = db.list_modules().await?;
                println!("{}", json!(modules));
            }
        },
        Commands::Task { command } => match command {
            TaskCommands::Create {
                module_id,
                name,
                description,
            } => {
                let task = Task {
                    id: None,
                    module_id,
                    name,
                    description,
                    status: "pending".to_string(),
                };
                let created = db.create_task(&task).await?;
                println!("{}", json!(created));
            }
            TaskCommands::List => {
                let tasks = db.list_tasks().await?;
                println!("{}", json!(tasks));
            }
        },
        Commands::Todo { command } => match command {
            TodoCommands::Create {
                project_id,
                content,
            } => {
                let todo = TodoItem {
                    id: None,
                    project_id,
                    task_id: None,
                    content,
                    status: "pending".to_string(),
                };
                let created = db.create_todo(&todo).await?;
                println!("{}", json!(created));
            }
            TodoCommands::List { project_id: _ } => {
                // TODO: Filter by project_id
                let todos = db.list_todos().await?;
                println!("{}", json!(todos));
            }
        },
        Commands::Context { task_id } => {
            let results = get_task_context(&task_id, &db, &vector_db).await?;
            println!("{}", json!({ "results": results }));
        }
    }

    Ok(())
}

fn print_error_json(kind: &str, message: String) {
    println!(
        "{}",
        json!({
            "status": "error",
            "kind": kind,
            "error": message
        })
    );
}
