use clap::Parser;

#[derive(Parser)]
pub(crate) struct Args {
    #[arg(long, default_value_t = 7700)]
    pub(crate) port: u16,
    #[arg(long)]
    pub(crate) no_open: bool,
}
