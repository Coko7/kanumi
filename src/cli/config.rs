use anyhow::Result;
use log::info;

use super::ConfigurationCommands;
use crate::{models::Configuration, utils};

pub fn handle_config_command(
    command: ConfigurationCommands,
    configuration: &Configuration,
) -> Result<()> {
    match command {
        ConfigurationCommands::Show => {
            let config_path = utils::common::get_config_file()?;
            let banner = utils::common::create_banner(&config_path.display().to_string());
            println!("{banner}");

            let toml_config = configuration.to_toml_str()?;
            println!("{toml_config}");
            Ok(())
        }
        ConfigurationCommands::Generate { dry_run: _ } => {
            info!("generating default config...");
            let default_config = Configuration::create_default();
            let toml = default_config.to_toml_str()?;
            print!("{}", toml);
            Ok(())
        }
    }
}
