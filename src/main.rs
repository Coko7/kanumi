use anyhow::{ensure, Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use log::{error, info, warn};

use models::Configuration;

mod cli;
mod models;
mod utils;

fn main() -> Result<()> {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("getting config file");
    let config_file = utils::common::get_config_file()?;
    if !config_file.exists() {
        utils::common::create_config_file()?;
        info!("config file created");
    }

    info!("loading config");
    let config = utils::common::load_config(config_file)?;

    info!("process cli args");
    match process_args(args, config) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("critical failure: {}", e);
            Err(e)
        }
    }
}

fn process_args(args: Cli, config: Configuration) -> Result<()> {
    let root_images_dir = config
        .root_images_dir
        .to_owned()
        .context("base directory must be set")?;

    ensure!(
        root_images_dir.exists(),
        "could not find root images directory: {}",
        root_images_dir.display()
    );

    let root_images_dir = config.root_images_dir.clone().unwrap();

    info!("metadata_path: {:?}", config.metadata_path);
    match args.command {
        Commands::List {
            score_filters,
            width_range,
            height_range,
            base_directory,
            ignore_config,
            use_json_format,
        } => {
            let mut score_filters = score_filters;
            let mut width_range = width_range;
            let mut height_range = height_range;

            if !ignore_config {
                score_filters = score_filters.or(config.score_filters);
                width_range = width_range.or(config.width_range);
                height_range = height_range.or(config.height_range);
            } else {
                info!("ignore_config flag has been added");
            }

            warn!("right now, metadata file is required to list images");
            cli::list_images_using_metadata(
                &root_images_dir,
                config.metadata_path,
                score_filters,
                width_range,
                height_range,
                base_directory,
                use_json_format,
            )
        }
        cli::Commands::Scan { use_json_format } => {
            cli::scan_images(&root_images_dir, config.metadata_path, use_json_format)
        }
        cli::Commands::Configuration { command } => cli::handle_config_command(command, &config),
        cli::Commands::Metadata { command } => cli::handle_metadata_command(command, &config),
    }
}
