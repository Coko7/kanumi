use std::{fs::File, ops::RangeInclusive, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use cli::{Cli, NodeType};
use log::{debug, info};
use model::{Configuration, ImageMeta};
use rand::{seq::SliceRandom, thread_rng};
use utils::{create_config_file, get_config_file, load_config};
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
    let config_file = get_config_file()?;
    if !config_file.exists() {
        create_config_file()?;
        info!("config file created");
    }

    info!("loading config");
    let config = load_config(config_file)?;

    info!("process cli args");
    process_args(args, config)?;

    Ok(())
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

fn is_shown(entry: &DirEntry, filter: NodeType) -> bool {
    match filter {
        NodeType::Directory => entry.file_type().is_dir(),
        NodeType::Image => is_image_file(entry),
    }
}

fn process_args(args: Cli, config: Configuration) -> Result<()> {
    let mut root_dir = config.root_images_dir;
    if let Some(cli_root) = args.directory {
        root_dir = cli_root;
    }

    let mut meta_file = config.metadata_path;
    if let Some(cli_meta) = args.metadata_path {
        meta_file = Some(cli_meta);
    }

    debug!("About to run WalkDir on {}", root_dir.display());
    let mut entries: Vec<_> = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| is_shown(entry, args.node_type))
        .collect();

    let mut rng = thread_rng();
    debug!("Shuffling entries");
    entries.shuffle(&mut rng);

    if let Some(count) = args.select_count {
        debug!("Taking first {} entries", count);
        entries = entries.into_iter().take(count).collect();
    }

    if let Some(meta) = meta_file {
        let mut records = create_meta_list(meta)?;

        if let Some(mut nsfw_filter) = args.score_range {
            if *nsfw_filter.end() > entries.len() {
                nsfw_filter = *nsfw_filter.start()..=entries.len();
            }

            records = records
                .into_iter()
                .filter(|record| score_matches(record, &nsfw_filter))
                .collect();
        }

        for entry in entries {
            if records.iter().any(|item| item.path == entry.path()) {
                println!("{}", entry.path().display())
            }
        }
    } else {
        for entry in entries {
            debug!("Printing entries");
            println!("{}", entry.path().display());
        }
    }

    Ok(())
}

fn score_matches(meta: &ImageMeta, range: &RangeInclusive<usize>) -> bool {
    usize::from(meta.score) >= *range.start() && usize::from(meta.score) <= *range.end()
}

fn create_meta_list(meta_file_path: PathBuf) -> Result<Vec<ImageMeta>, anyhow::Error> {
    let mut reader = csv::Reader::from_reader(File::open(meta_file_path)?);

    let mut records = Vec::new();
    for entry in reader.deserialize() {
        let record: ImageMeta = entry?;
        records.push(record);
    }

    Ok(records)
}
