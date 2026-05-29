mod args;
mod context;
mod epic;
mod file;
mod issue;
mod knowledge;
mod link;
mod module;
mod persona;
mod project;
mod task;
mod todo;
mod user_story;
mod utils;
mod workflow;

use clap::Parser;
use utils::print_error_json;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let args = match args::Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
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

async fn run(args: args::Args) -> anyhow::Result<()> {
    let config = dn_core::config::Config::load()?;

    if let args::Commands::Config { command } = &args.command {
        return handle_config_command(command, &config);
    }

    let db = dn_core::db::DB::from_config(&config).await?;
    dispatch_command(args.command, &db, args.pretty, args.ignore_case).await
}

fn handle_config_command(
    command: &args::ConfigCommands,
    config: &dn_core::config::Config,
) -> anyhow::Result<()> {
    match command {
        args::ConfigCommands::Show => {
            print!("{}", config);
        }
    }
    Ok(())
}

async fn dispatch_command(
    command: args::Commands,
    db: &dn_core::db::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::Commands::Add {
            field_names,
            field_values,
            link_to,
        } => knowledge::handle_add(field_names, field_values, link_to, db, pretty).await,
        args::Commands::Link {
            from_id,
            edge,
            to_id,
        } => link::handle_link(from_id, edge, to_id, db, pretty).await,
        args::Commands::Project { command } => {
            project::handle_project_command(command, db, pretty).await
        }
        args::Commands::Module { command } => {
            module::handle_module_command(command, db, pretty, ignore_case).await
        }
        args::Commands::File { command } => {
            file::handle_file_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Task { command } => {
            task::handle_task_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Todo { command } => {
            todo::handle_todo_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Issue { command } => {
            issue::handle_issue_command(command, db, pretty, ignore_case).await
        }
        args::Commands::UserStory { command } => {
            user_story::handle_user_story_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Epic { command } => {
            epic::handle_epic_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Persona { command } => {
            persona::handle_persona_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Workflow { command } => {
            workflow::handle_workflow_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Context {
            task_id,
            file_id,
            epic_id,
            full,
            general,
            project,
        } => {
            context::handle_context(
                task_id, file_id, epic_id, full, general, project, db, pretty,
            )
            .await
        }
        args::Commands::Rm { context_ids } => knowledge::handle_rm(context_ids, db, pretty).await,
        args::Commands::Purge => context::handle_purge(db, pretty).await,
        args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}
