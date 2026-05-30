mod args;
mod cmd;
mod config;
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
        return config::config_show(command, &config);
    }

    let db = dn_core::db::surreal::DB::from_config(&config).await?;
    cmd::dispatch(args.command, &db, args.pretty, args.ignore_case).await
}
