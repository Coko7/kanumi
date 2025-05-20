use anyhow::{bail, Context, Result};
use log::info;
use std::path::Path;

use crate::{
    models::{Configuration, ImageMeta},
    utils,
};

use super::MetadataCommands;

pub fn handle_metadata_command(
    command: MetadataCommands,
    configuration: &Configuration,
) -> Result<()> {
    let metadata_path = configuration
        .metadata_path
        .clone()
        .context("metadata path must be set")?;

    let all_metadatas = utils::common::load_image_metas(metadata_path)?;

    match command {
        MetadataCommands::Show => {
            let metas_json = serde_json::to_string(&all_metadatas)?;
            Ok(println!("{metas_json}"))
        }
        MetadataCommands::Get { image } => get_metadata(&image, &all_metadatas),
        MetadataCommands::Edit {
            image: _,
            metadata: _,
        } => todo!(),
        MetadataCommands::Generate { image, dry_run: _ } => {
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
        MetadataCommands::Search { query: _ } => todo!(),
    }
}

fn get_metadata(image: &Path, metadatas: &[ImageMeta]) -> Result<()> {
    if !image.exists() {
        bail!("no such image: {}", image.display());
    }

    if !image.is_file() {
        bail!("the following is not a valid image: {}", image.display());
    }

    let hash = utils::common::compute_blake3_hash(image)?;

    let result = metadatas.iter().find(|meta| meta.id == hash);
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
