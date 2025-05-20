use anyhow::{bail, Result};
use log::info;
use std::{
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use crate::{models::ScoreFilter, utils};

pub fn list_images_using_metadata(
    root_images_dir: &Path,
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

    let mut metas = utils::common::load_image_metas(metadata_path.unwrap())?;

    info!("score_filters: {:?}", score_filters);
    info!("width_range: {:?}", width_range);
    info!("height_range: {:?}", height_range);

    if let Some(base_directory) = base_directory {
        info!("filter using base_directory: {:?}", base_directory);
        metas.retain(|meta| {
            let base_directory = if base_directory.is_absolute() {
                base_directory.clone()
            } else {
                root_images_dir.join(&base_directory)
            };

            if let Some(img_base_dir) = meta.path.parent() {
                img_base_dir == base_directory
            } else {
                false
            }
        });
    }

    if width_range.is_some() || height_range.is_some() {
        info!("applying dimensions filter...");
        metas.retain(|meta| {
            utils::common::image_matches_dims(&meta.path, &width_range, &height_range)
        });
    }

    if let Some(score_filters) = score_filters {
        info!("applying image meta score filters...");

        for score_filter in score_filters.iter() {
            metas.retain(|meta| utils::common::image_score_matches(meta, score_filter));
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
    let mut images = utils::common::get_all_images(&base_directory)?;

    if width_range.is_some() || height_range.is_some() {
        info!("applying dimensions filter...");
        images.retain(|img| utils::common::image_matches_dims(img, &width_range, &height_range));
    }

    for image in images.iter() {
        println!("{}", image.display());
    }

    todo!("not fully supported yet!")
}
