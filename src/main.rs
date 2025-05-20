use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use log::{error, info, warn};

use cli::Cli;
use config::Configuration;
use image_meta::ImageMeta;

mod cli;
mod config;
mod image_meta;
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
        cli::Commands::List {
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
            utils::list::list_images_using_metadata(
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
            utils::scan::scan_images(&root_images_dir, config.metadata_path, use_json_format)
        }
        cli::Commands::Configuration { command } => match command {
            cli::ConfigurationCommands::Show => {
                let config_path = utils::common::get_config_file()?;
                let banner = utils::common::create_banner(&config_path.display().to_string());
                println!("{banner}");

                let toml_config = config.to_toml_str()?;
                println!("{toml_config}");
                Ok(())
            }
            cli::ConfigurationCommands::Generate { dry_run: _ } => {
                info!("generating default config...");
                let default_config = Configuration::create_default();
                let toml = default_config.to_toml_str()?;
                print!("{}", toml);
                Ok(())
            }
        },
        cli::Commands::Metadata { command } => match command {
            cli::MetadataCommands::Show => {
                let metas = utils::common::load_image_metas(config.metadata_path.unwrap())?;
                let metas_json = serde_json::to_string(&metas)?;
                println!("{metas_json}");
                Ok(())
            }
            cli::MetadataCommands::Get { image } => {
                if !image.exists() {
                    bail!("no such image: {}", image.display());
                }

                if !image.is_file() {
                    bail!("the following is not a valid image: {}", image.display());
                }

                let hash = utils::common::compute_blake3_hash(&image)?;
                let metas = utils::common::load_image_metas(config.metadata_path.unwrap())?;

                let result = metas.iter().find(|meta| meta.id == hash);
                match result {
                    Some(meta) => {
                        let meta_json = serde_json::to_string(meta)?;
                        println!("{meta_json}");
                        Ok(())
                    }
                    None => {
                        bail!("no matching metadata for image: {}", image.display())
                    }
                }
            }
            cli::MetadataCommands::Generate { image, dry_run: _ } => {
                info!("generating default metadata...");
                let meta = ImageMeta::create_from_image(&image)?;
                let json = serde_json::to_string(&meta)?;
                println!("{}", json);
                Ok(())

                // let images = get_all_images(&base_dir)?;
                // let mut metadatas = vec![];
                // for image in images.iter() {
                //     let meta = ImageMeta::create_from_image(image)?;
                //     metadatas.push(meta);
                // }
                //
                // let json = serde_json::to_string(&metadatas)?;
                // println!("{}", json);
                // Ok(())
            }
            cli::MetadataCommands::Edit {
                image: _,
                metadata: _,
            } => todo!(),
        },
    }
}
