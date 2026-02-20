use clap::Parser;
use clap::error::ErrorKind;
use lazydev::args::{
    Args, Commands, ConfigCommands, ModuleCommands, ProjectCommands, TaskCommands, TodoCommands,
};
use lazydev::config::Config;
use lazydev::context::get_task_context;
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::models::{Module, Project, Task, TaskStatus, TaskUpdate, TodoItem};
use lazydev::vector_db::VectorDB;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
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
    let config = Config::load(args.backend.as_deref())?;
    if let Commands::Config { command } = &args.command {
        match command {
            ConfigCommands::Show => {
                println!("{}", config.redacted_json());
            }
        }
        return Ok(());
    }

    let db = DB::from_config(&config).await?;
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
                    status: TaskStatus::NotStarted,
                };
                let created = db.create_task(&task).await?;
                println!("{}", json!(created));
            }
            TaskCommands::Update {
                task_id,
                name,
                description,
                status,
            } => {
                let parsed_status = match status {
                    Some(value) => Some(TaskStatus::parse(&value).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid status '{}'. Expected one of: not_started, started, finished",
                            value
                        )
                    })?),
                    None => None,
                };
                let updated = db
                    .update_task(&task_id, name, description, parsed_status)
                    .await?;
                if let Some(task) = updated {
                    println!("{}", json!(task));
                } else {
                    return Err(anyhow::anyhow!("Task not found: {}", task_id));
                }
            }
            TaskCommands::AppendUpdate { task_id, content } => {
                if db.get_task(&task_id).await?.is_none() {
                    return Err(anyhow::anyhow!("Task not found: {}", task_id));
                }

                let created_at_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| anyhow::anyhow!("System clock is before UNIX_EPOCH"))?
                    .as_millis() as i64;

                let update = TaskUpdate {
                    id: None,
                    task_id,
                    content,
                    created_at_ms,
                    updated_at_ms: None,
                };
                let created = db.create_task_update(&update).await?;
                println!("{}", json!(created));
            }
            TaskCommands::UpdateEntry { update_id, content } => {
                let updated_at_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| anyhow::anyhow!("System clock is before UNIX_EPOCH"))?
                    .as_millis() as i64;
                let edited = db
                    .update_task_update(&update_id, content, updated_at_ms)
                    .await?;
                if let Some(update) = edited {
                    println!("{}", json!(update));
                } else {
                    return Err(anyhow::anyhow!("Task update not found: {}", update_id));
                }
            }
            TaskCommands::ListUpdates { task_id } => {
                let updates = db.list_task_updates(&task_id).await?;
                println!("{}", json!(updates));
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
        Commands::Config { .. } => unreachable!("config command returns before DB init"),
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
