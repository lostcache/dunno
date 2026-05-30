use crate::args;

pub(crate) fn config_show(
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
