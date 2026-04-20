mod args;
mod commands;
mod entities;

use clap::Parser;

use commands::*;

/// Application entry point.
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

/// Main application logic dispatcher.
async fn run(args: args::Args) -> anyhow::Result<()> {
    let config = dn_core::config::Config::load()?;

    if let args::Commands::Config { command } = &args.command {
        return handle_config_command(command, &config, args.pretty);
    }

    let db = dn_core::db::DB::from_config(&config).await?;
    dispatch_command(args.command, &db, args.pretty, args.ignore_case).await
}

/// Routes commands to their specialized handlers.
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
        } => handle_add(field_names, field_values, link_to, db, pretty).await,
        args::Commands::Link {
            from_id,
            edge,
            to_id,
        } => handle_link(from_id, edge, to_id, db, pretty).await,
        args::Commands::Project { command } => handle_project_command(command, db, pretty).await,
        args::Commands::Module { command } => {
            handle_module_command(command, db, pretty, ignore_case).await
        }
        args::Commands::File { command } => {
            handle_file_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Task { command } => {
            handle_task_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Todo { command } => {
            entities::todo::handle_todo_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Issue { command } => {
            entities::issue::handle_issue_command(command, db, pretty).await
        }
        args::Commands::UserStory { command } => {
            entities::user_story::handle_user_story_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Epic { command } => {
            entities::epic::handle_epic_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Persona { command } => {
            entities::persona::handle_persona_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Workflow { command } => {
            entities::workflow::handle_workflow_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Context {
            task_id,
            file_id,
            epic_id,
            full,
            general,
            project,
        } => {
            entities::context::handle_context(
                task_id, file_id, epic_id, full, general, project, db, pretty,
            )
            .await
        }
        args::Commands::Rm { context_ids } => handle_rm(context_ids, db, pretty).await,
        args::Commands::Purge => entities::context::handle_purge(db, pretty).await,
        args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}
