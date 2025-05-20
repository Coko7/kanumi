use anyhow::{bail, Result};
use log::{debug, info, warn};
use serde_json::json;
use std::{collections::HashMap, path::PathBuf};

use crate::{models::ImageMeta, utils};

pub fn scan_images(
    base_directory: &PathBuf,
    metadata_path: Option<PathBuf>,
    use_json_format: bool,
) -> Result<()> {
    if metadata_path.is_none() {
        bail!("No metadata file provided!");
    }

    info!("scanning for missing metadata or images...");

    info!("about to run WalkDir on {}", base_directory.display());
    let all_metas = utils::common::load_image_metas(metadata_path.unwrap())?;

    let mut mappings: HashMap<&PathBuf, Option<ImageMeta>> = HashMap::new();
    let images = utils::common::get_all_images(base_directory)?;
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
            let hash = utils::common::compute_blake3_hash(img_path)?;
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
            warn!("cannot find image: {}", meta.path.display());
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
                println!("{} new:", new_images.len());
                for (_, img_path) in new_images.iter() {
                    println!("- {}", img_path.display());
                }
                println!();
            }

            if !moved_images.is_empty() {
                println!("{} moved:", moved_images.len());
                for (new_path, metadata) in moved_images.iter() {
                    println!("- {} -> {}", metadata.path.display(), new_path.display())
                }
                println!();
            }

            if !deleted_images.is_empty() {
                println!("{} deleted:", deleted_images.len());
                for metadata in deleted_images.iter() {
                    println!("- {}", metadata.path.display());
                }
                println!();
            }
        }
    }

    Ok(())
}
