use clap::Parser;
use clap::error::ErrorKind;
use lazydev::args::{
    Args, Commands, ConfigCommands, FileCommands, ModuleCommands, ProjectCommands,
    SubmoduleCommands, SubtaskCommands, TaskCommands, TodoCommands,
};
use lazydev::config::Config;
use lazydev::context::{get_file_context, get_subtask_context, get_task_context};
use lazydev::db::DB;
use lazydev::ingest::add_knowledge;
use lazydev::models::Project;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

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

    match args.command {
        Commands::Add {
            kind,
            content,
            link_to,
        } => {
            add_knowledge(kind, content, link_to, &db).await?;
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
                let created = db.create_module(&name, &description, &project_id).await?;
                println!("{}", json!(created));
            }
            ModuleCommands::List => {
                let modules = db.list_modules().await?;
                println!("{}", json!(modules));
            }
        },
        Commands::Submodule { command } => match command {
            SubmoduleCommands::Create {
                module_id,
                name,
                description,
            } => {
                let created =
                    db.create_submodule(&name, &description, &module_id).await?;
                println!("{}", json!(created));
            }
            SubmoduleCommands::List { module_id } => {
                let submodules = if let Some(mid) = module_id {
                    db.list_submodules_by_module(&mid).await?
                } else {
                    db.list_submodules().await?
                };
                println!("{}", json!(submodules));
            }
        },
        Commands::File { command } => match command {
            FileCommands::Create {
                parent_id,
                name,
                path,
            } => {
                let created = db.create_file(&name, &path, &parent_id).await?;
                println!("{}", json!(created));
            }
            FileCommands::List {
                module_id,
                submodule_id,
            } => {
                let files = if let Some(mid) = module_id {
                    db.list_files_by_module(&mid).await?
                } else if let Some(sid) = submodule_id {
                    db.list_files_by_submodule(&sid).await?
                } else {
                    db.list_files().await?
                };
                println!("{}", json!(files));
            }
        },
        Commands::Task { command } => match command {
            TaskCommands::Create {
                module_id,
                name,
                description,
            } => {
                let created = db.create_task(&name, &description, &module_id).await?;
                println!("{}", json!(created));
            }
            TaskCommands::Update {
                task_id,
                name,
                description,
                status,
            } => {
                let parsed_status = match status {
                    Some(value) => {
                        Some(lazydev::models::TaskStatus::parse(&value).ok_or_else(
                            || {
                                anyhow::anyhow!(
                                    "Invalid status '{}'. Expected: not_started, started, finished",
                                    value
                                )
                            },
                        )?)
                    }
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

                let created =
                    db.create_task_update(&content, created_at_ms, &task_id).await?;
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
                    return Err(anyhow::anyhow!(
                        "Task update not found: {}",
                        update_id
                    ));
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
        Commands::Subtask { command } => match command {
            SubtaskCommands::Create {
                task_id,
                name,
                description,
            } => {
                let created =
                    db.create_subtask(&name, &description, &task_id).await?;
                println!("{}", json!(created));
            }
            SubtaskCommands::List { task_id } => {
                let subtasks = db.list_subtasks_by_task(&task_id).await?;
                println!("{}", json!(subtasks));
            }
        },
        Commands::Todo { command } => match command {
            TodoCommands::Create {
                project_id,
                content,
            } => {
                let created = db.create_todo(&content, &project_id).await?;
                println!("{}", json!(created));
            }
            TodoCommands::List { project_id } => {
                let todos = db.list_todos_by_project(&project_id).await?;
                println!("{}", json!(todos));
            }
        },
        Commands::Context {
            task_id,
            file_id,
            subtask_id,
        } => {
            if let Some(t_id) = task_id {
                let results = get_task_context(&t_id, &db).await?;
                println!("{}", json!({ "results": results }));
            } else if let Some(f_id) = file_id {
                let results = get_file_context(&f_id, &db).await?;
                println!("{}", json!({ "results": results }));
            } else if let Some(st_id) = subtask_id {
                let results = get_subtask_context(&st_id, &db).await?;
                println!("{}", json!({ "results": results }));
            } else {
                return Err(anyhow::anyhow!(
                    "One of --task-id, --file-id, or --subtask-id must be provided"
                ));
            }
        }
        Commands::Config { .. } => {
            unreachable!("config command returns before DB init")
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
