use std::path::Path;

use anyhow::Result;
use log::info;

use super::ConfigurationCommands;
use crate::{models::Configuration, utils};

pub fn handle_config_command(
    command: ConfigurationCommands,
    configuration: &Configuration,
) -> Result<()> {
    match command {
        ConfigurationCommands::Show(display_format) => {
            let config_path = utils::common::get_config_file()?;

            if display_format.json {
                show_config_as_json(configuration)
            } else if display_format.toml {
                show_config_as_toml(configuration, &config_path)
            } else {
                show_config_as_toml(configuration, &config_path)
            }
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

fn show_config_as_json(configuration: &Configuration) -> Result<()> {
    let json_config = serde_json::to_string(configuration)?;
    println!("{json_config}");
    Ok(())
}

fn show_config_as_toml(configuration: &Configuration, config_path: &Path) -> Result<()> {
    let banner = utils::common::create_banner(&config_path.display().to_string());
    println!("{banner}");

    let toml_config = configuration.to_toml_str()?;
    println!("{toml_config}");
    Ok(())
}
