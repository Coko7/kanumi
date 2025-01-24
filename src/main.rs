use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cli::{Cli, NodeType};
use log::{debug, error, info};
use model::{Configuration, ImageMeta};
use walkdir::{DirEntry, WalkDir};

mod cli;
mod model;
mod utils;

fn main() -> Result<()> {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("getting config file");
    let config_file = utils::get_config_file()?;
    if !config_file.exists() {
        utils::create_config_file()?;
        info!("config file created");
    }

    info!("loading config");
    let config = utils::load_config(config_file)?;

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
    if args.generate_config {
        info!("generating default config...");
        let default_config = Configuration::create_default();
        let toml = default_config.to_toml_str()?;
        print!("{}", toml);
        return Ok(());
    }

    match args.node_type {
        NodeType::Directory => process_only_dirs(&args, &config),
        NodeType::Image => process_only_images(&args, &config),
    }
}

fn is_image_file(entry: &DirEntry) -> bool {
    if let Some(file_name) = entry.file_name().to_str() {
        return file_name.to_lowercase().ends_with(".gif")
            || file_name.to_lowercase().ends_with(".jpeg")
            || file_name.to_lowercase().ends_with(".jpg")
            || file_name.to_lowercase().ends_with(".png")
            || file_name.to_lowercase().ends_with(".webp");
    }

    false
}

fn process_only_dirs(args: &Cli, config: &Configuration) -> Result<()> {
    let root_dir = args.directory.clone().unwrap_or(
        config
            .root_images_dir
            .clone()
            .expect("root dir not provided!"),
    );

    if !root_dir.exists() {
        return Err(anyhow!("could not find directory: {}", root_dir.display()));
    }

    debug!("About to run WalkDir on {}", root_dir.display());

    let dirs: Vec<_> = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .collect();

    for dir in dirs {
        println!("{}", dir.path().display());
    }

    Ok(())
}

fn process_only_images(args: &Cli, config: &Configuration) -> Result<()> {
    let dir_path = args
        .directory
        .clone()
        .or(config.root_images_dir.clone())
        .context("Root directory must be specified")?;

    if !dir_path.exists() {
        return Err(anyhow!("could not find directory: {}", dir_path.display()));
    }

    info!("processing args/config...");

    let metadata_path = args.metadata_path.clone().or(config.metadata_path.clone());
    info!("metadata_path: {:?}", metadata_path);

    let score_range = args.score_range.clone().or(config.score_range.clone());
    info!("score_range: {:?}", score_range);

    let width_range = args.width_range.clone().or(config.width_range.clone());
    info!("width_range: {:?}", width_range);

    let height_range = args.height_range.clone().or(config.height_range.clone());
    info!("height_range: {:?}", height_range);

    info!("about to run WalkDir on {}", dir_path.display());
    let mut images: Vec<_> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|node| is_image_file(node))
        .map(|entry| entry.path().to_owned())
        .collect();

    if width_range.is_some() || height_range.is_some() {
        info!("applying dimensions filter...");
        images.retain(|img| utils::image_matches_dims(img, &width_range, &height_range));
    }

    if let Some(score_range) = score_range {
        if metadata_path.is_none() {
            return Err(anyhow!("No metadata file provided!"));
        }

        info!("applying image meta score filter...");
        let metadata_path = metadata_path.unwrap();
        let records = utils::create_meta_list(metadata_path)?;

        let filtered_records: Vec<ImageMeta> = records
            .into_iter()
            .filter(|record| utils::image_score_matches(record, &score_range))
            .collect();

        images.retain(|img| filtered_records.iter().any(|record| record.path == *img));
    }

    for image in images.iter() {
        println!("{}", image.display());
    }

    Ok(())
}
