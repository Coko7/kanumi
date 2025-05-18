use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use log::{debug, error, info, warn};
use serde_json::json;
use std::{collections::HashMap, ops::RangeInclusive, path::PathBuf};

use cli::{Cli, ScoreFilter};
use config::Configuration;
use image_meta::ImageMeta;
use utils::get_all_images;

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
        cli::Commands::Filter {
            score_filters,
            width_range,
            height_range,
            base_directory,
            use_json_format,
        } => {
            let score_filters = score_filters.or(config.score_filters);
            let width_range = width_range.or(config.width_range);
            let height_range = height_range.or(config.height_range);

            warn!("right now, metadata file is required to filter images");
            filter_images_using_metadata(
                &root_images_dir,
                config.metadata_path,
                score_filters,
                width_range,
                height_range,
                base_directory,
                use_json_format,
            )
        }
        cli::Commands::Query { image } => {
            if !image.exists() {
                bail!("no such image: {}", image.display());
            }

            if !image.is_file() {
                bail!("the following is not a valid image: {}", image.display());
            }

            let hash = utils::compute_blake3_hash(&image)?;
            let metas = utils::load_image_metas(config.metadata_path.unwrap())?;

            let result = metas.iter().find(|meta| meta.id == hash);
            match result {
                Some(meta) => {
                    println!("{:?}", meta);
                    Ok(())
                }
                None => {
                    bail!("no matching metadata for image: {}", image.display())
                }
            }
        }
        cli::Commands::Scan { use_json_format } => {
            scan_images(&root_images_dir, config.metadata_path, use_json_format)
        }
        cli::Commands::Configuration { command } => match command {
            cli::ConfigurationCommands::Show => {
                let config_path = utils::get_config_file()?;
                let banner = utils::create_banner(&config_path.display().to_string());
                println!("{banner}");

                let toml_config = config.to_toml_str()?;
                println!("{toml_config}");
                Ok(())
            }
            cli::ConfigurationCommands::Generate { dry_run } => {
                info!("generating default config...");
                let default_config = Configuration::create_default();
                let toml = default_config.to_toml_str()?;
                print!("{}", toml);
                Ok(())
            }
        },
        cli::Commands::Metadata { command } => match command {
            cli::MetadataCommands::Generate { image, dry_run } => {
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
            cli::MetadataCommands::Show => todo!(),
        },
    }
}

fn scan_images(
    base_directory: &PathBuf,
    metadata_path: Option<PathBuf>,
    use_json_format: bool,
) -> Result<()> {
    if metadata_path.is_none() {
        bail!("No metadata file provided!");
    }

    info!("scanning for missing metadata or images...");

    info!("about to run WalkDir on {}", base_directory.display());
    let all_metas = utils::load_image_metas(metadata_path.unwrap())?;

    let mut mappings: HashMap<&PathBuf, Option<ImageMeta>> = HashMap::new();
    let images = get_all_images(&base_directory)?;
    for image_path in images.iter() {
        let matching_meta = all_metas
            .iter()
            .find(|meta| meta.path == *image_path)
            .cloned();

        mappings.insert(image_path, matching_meta);
    }

    debug!("created {} img:Option<meta> mappings", mappings.len());

    let mut metaless_images: HashMap<String, &PathBuf> = HashMap::new();
    for (img_path, metadata) in mappings.iter() {
        if metadata.is_none() {
            let hash = utils::compute_blake3_hash(img_path)?;
            metaless_images.insert(hash, img_path);
        }
    }

    debug!(
        "computed hash for {} images that had no metadata",
        metaless_images.len()
    );

    let mut moved_images: HashMap<&PathBuf, &ImageMeta> = HashMap::new();
    let mut deleted_images: Vec<&ImageMeta> = vec![];

    for meta in all_metas.iter() {
        if meta.path.exists() {
            continue;
        }

        warn!("image path invalid for: {meta:?}");
        if let Some(image_path) = metaless_images.get(&meta.id) {
            warn!(
                "{} seems to have been moved to: {}",
                meta.path.display(),
                image_path.display()
            );
            moved_images.insert(image_path, meta);
            metaless_images.retain(|hash, _| *hash != meta.id);
        } else {
            error!("cannot find image: {}", meta.path.display());
            deleted_images.push(meta);
        }
    }

    let new_images = metaless_images;

    match use_json_format {
        true => {
            let new_images: Vec<_> = new_images.values().collect();
            let moved_images: Vec<_> = moved_images
                .iter()
                .map(|(new_path, meta)| {
                    json!({
                        "metadata": meta,
                        "new_path": new_path
                    })
                })
                .collect();

            let summary = json!({
                "new": new_images,
                "moved": moved_images,
                "deleted": deleted_images,
            });
            let summary_json = serde_json::to_string(&summary)?;
            println!("{summary_json}");
        }
        false => {
            if !new_images.is_empty() {
                println!("new:");
                for (_, img_path) in new_images.iter() {
                    println!("- {}", img_path.display());
                }
                println!();
            }

            if !moved_images.is_empty() {
                println!("moved:");
                for (new_path, metadata) in moved_images.iter() {
                    println!("- {} -> {}", metadata.path.display(), new_path.display())
                }
                println!();
            }

            if !deleted_images.is_empty() {
                println!("deleted:");
                for metadata in deleted_images.iter() {
                    println!("- {}", metadata.path.display());
                }
                println!();
            }
        }
    }

    Ok(())
}

fn filter_images_using_metadata(
    root_images_dir: &PathBuf,
    metadata_path: Option<PathBuf>,
    score_filters: Option<Vec<ScoreFilter>>,
    width_range: Option<RangeInclusive<usize>>,
    height_range: Option<RangeInclusive<usize>>,
    base_directory: Option<PathBuf>,
    use_json_format: bool,
) -> Result<()> {
    if metadata_path.is_none() {
        bail!("No metadata file provided!");
    }

    let mut metas = utils::load_image_metas(metadata_path.unwrap())?;

    info!("score_filters: {:?}", score_filters);
    info!("width_range: {:?}", width_range);
    info!("height_range: {:?}", height_range);

    if let Some(base_directory) = base_directory {
        info!("filter using base_directory: {:?}", base_directory);
        metas.retain(|meta| {
            assert!(
                meta.path.is_file(),
                "{} should be an image!",
                meta.path.display()
            );

            let base_directory = if base_directory.is_absolute() {
                base_directory.clone()
            } else {
                root_images_dir.join(&base_directory)
            };

            if let Some(img_base_dir) = meta.path.parent() {
                img_base_dir == &base_directory
            } else {
                false
            }
        });
    }

    if width_range.is_some() || height_range.is_some() {
        info!("applying dimensions filter...");
        metas.retain(|meta| utils::image_matches_dims(&meta.path, &width_range, &height_range));
    }

    if let Some(score_filters) = score_filters {
        info!("applying image meta score filters...");

        for score_filter in score_filters.iter() {
            metas.retain(|meta| utils::image_score_matches(meta, score_filter));
        }
    }

    match use_json_format {
        true => {
            info!("outputting as json");
            let metas_json = serde_json::to_string(&metas)?;
            println!("{}", metas_json);
        }
        false => {
            info!("outputting image paths only");
            for meta in metas.iter() {
                println!("{}", meta.path.display());
            }
        }
    };

    Ok(())
}

fn filter_images_without_using_metadata(
    base_directory: PathBuf,
    width_range: Option<RangeInclusive<usize>>,
    height_range: Option<RangeInclusive<usize>>,
) -> Result<()> {
    info!("width_range: {:?}", width_range);
    info!("height_range: {:?}", height_range);

    info!("about to run WalkDir on {}", base_directory.display());
    let mut images = get_all_images(&base_directory)?;

    if width_range.is_some() || height_range.is_some() {
        info!("applying dimensions filter...");
        images.retain(|img| utils::image_matches_dims(img, &width_range, &height_range));
    }

    for image in images.iter() {
        println!("{}", image.display());
    }

    todo!("not fully supported yet!")
}
