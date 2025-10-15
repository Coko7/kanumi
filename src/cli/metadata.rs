use anyhow::{bail, Context, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::{debug, info};
use std::ffi::OsString;

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
            println!("{metas_json}");
            Ok(())
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
            let result = search_metadata(configuration, query, &metadatas)?;
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
            }
            Ok(())
        }
    }
}

fn search_metadata(
    configuration: &Configuration,
    query: OsString,
    metadatas: &[ImageMeta],
) -> Result<Option<ImageMeta>> {
    let query = query.to_string_lossy().into_owned();
    let matcher = SkimMatcherV2::default();
    let mut best_score = -1;
    let mut meta_scores: Vec<(&ImageMeta, i64)> = Vec::new();

    for meta in metadatas.iter() {
        // Test for exact match
        if meta.path.to_str().context("meta path should be a string")? == query {
            return Ok(Some(meta.clone()));
        }

        let root_images_dir = configuration
            .root_images_dir
            .to_str()
            .context("root images dir should be a valid string")?;

        let local_meta_path_str = meta
            .path
            .to_str()
            .context("meta path should be a string")?
            .strip_prefix(root_images_dir)
            .context("meta path should have root path prefix")?;

        if let Some(score) = matcher.fuzzy_match(local_meta_path_str, &query) {
            debug!("fuzzy: score for `{}`: `{}`", meta.path.display(), score);

            meta_scores.push((meta, score));
            if score > best_score {
                debug!("fuzzy: update best score: `{}` -> `{}`", best_score, score);
                best_score = score;
            }
        }
    }

    let best_candidates: Vec<_> = meta_scores
        .iter()
        .filter(|tuple| tuple.1 == best_score)
        .collect();

    // More than one candidate
    if best_candidates.len() > 1 {
        debug!("matches: {}", best_candidates.len());

        for candidate in best_candidates.iter() {
            debug!("{} -> {}", candidate.0.path.display(), candidate.1);
        }

        bail!(
            "too many matches for `{}`: {} results",
            query,
            best_candidates.len()
        );
    }

    if let Some(best_match) = best_candidates.first() {
        return Ok(Some(best_match.0.clone()));
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
