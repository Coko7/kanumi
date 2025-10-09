use anyhow::{bail, Context, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::{debug, error, info};
use std::{ffi::OsString, path::Path};

use crate::{
    models::{Configuration, ImageMeta},
    utils,
};

use super::MetadataCommands;

pub fn handle_metadata_command(
    command: MetadataCommands,
    configuration: &Configuration,
) -> Result<()> {
    let metadatas = utils::common::load_image_metas(&configuration.metadata_path)?;

    match command {
        MetadataCommands::Show => {
            let metas_json = serde_json::to_string(&metadatas)?;
            Ok(println!("{metas_json}"))
        }
        MetadataCommands::Get { identifier } => get_metadata(&identifier, &metadatas),
        MetadataCommands::Edit {
            identifier,
            payload,
        } => update_metadata(&identifier, &payload, &metadatas),
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
        MetadataCommands::Search {
            query,
            use_json_format,
        } => {
            let result = search_metadata(query, &metadatas)?;
            if let Some(metadata) = result {
                match use_json_format {
                    true => {
                        info!("outputting as json");
                        let metas_json = serde_json::to_string(&metadata)?;
                        println!("{}", metas_json);
                    }
                    false => {
                        info!("outputting image path only");
                        println!("{}", metadata.path.display());
                    }
                };
                println!("{}", metadata.path.display());
            }
            Ok(())
        }
    }
}

fn search_metadata(query: OsString, metadatas: &[ImageMeta]) -> Result<Option<&ImageMeta>> {
    let query = query.to_string_lossy().into_owned();
    let path = Path::new(&query);
    let matcher = SkimMatcherV2::default();
    let mut best_score = -1;
    let mut meta_scores: Vec<(&ImageMeta, i64)> = Vec::new();

    for meta in metadatas.iter() {
        if meta.path == path {
            return Ok(Some(meta));
        }

        let meta_path_str = meta.path.to_str().context("string expected here bro")?;
        if let Some(score) = matcher.fuzzy_match(meta_path_str, &query) {
            debug!("fuzzy: score for `{}`: `{}`", meta.path.display(), score);

            meta_scores.push((meta, score));
            if score > best_score {
                debug!("fuzzy: update best score: `{}` -> `{}`", best_score, score);
                best_score = score;
            }
        }
    }

    let mut best_candidates = meta_scores.iter().filter(|tuple| tuple.1 == best_score);

    // More than one candidate
    if best_candidates.clone().count() > 1 {
        error!("fuzzy: multiple best candidates: {:#?}", best_candidates);

        let candidates_labels: String = best_candidates
            .map(|tuple| format!("`{}`", tuple.0.path.to_str().unwrap()))
            .collect::<Vec<String>>()
            .join(", ");

        bail!(
            "Failed to find a single best match, there are several candidates: {}",
            candidates_labels
        );
    }

    if let Some(best_match) = best_candidates.next() {
        return Ok(Some(best_match.0));
    }

    Ok(None)
}

fn update_metadata(
    identifier: &OsString,
    _payload: &OsString,
    metadatas: &[ImageMeta],
) -> Result<()> {
    let identifier = identifier.to_string_lossy();
    let meta = utils::common::get_image_by_path_or_id(&identifier, metadatas)?;
    if meta.is_none() {
        bail!("no matching metadata for: {identifier}");
    }

    // let meta = meta.unwrap();
    todo!()
}

fn get_metadata(identifier: &OsString, metadatas: &[ImageMeta]) -> Result<()> {
    let identifier = identifier.to_string_lossy();
    match utils::common::get_image_by_path_or_id(&identifier, metadatas)? {
        Some(meta) => {
            let meta_json = serde_json::to_string(meta)?;
            println!("{meta_json}");
            Ok(())
        }
        None => {
            bail!("no matching metadata for: {identifier}")
        }
    }
}
