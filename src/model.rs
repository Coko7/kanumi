use anyhow::Result;
use directories::UserDirs;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct ImageMeta {
    #[serde(rename = "image_path")]
    pub path: PathBuf,

    #[serde(rename = "score")]
    pub score: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(rename = "root_path")]
    pub root_images_dir: Option<PathBuf>,

    #[serde(rename = "meta_path")]
    pub metadata_path: Option<PathBuf>,

    #[serde(rename = "score")]
    pub score_range: Option<RangeInclusive<usize>>,

    #[serde(rename = "width")]
    pub width_range: Option<RangeInclusive<usize>>,

    #[serde(rename = "height")]
    pub height_range: Option<RangeInclusive<usize>>,
}

impl Configuration {
    pub fn create_default() -> Configuration {
        let mut default_root_dir = None;
        let mut default_meta_path = None;

        if let Some(user_dirs) = UserDirs::new() {
            if let Some(picture_dir) = user_dirs.picture_dir() {
                info!("found user pictures dir: {}", picture_dir.display());
                default_root_dir = Some(picture_dir.to_path_buf());
                default_meta_path = Some(picture_dir.join("metadata.csv"));
            }
        }

        Configuration {
            root_images_dir: default_root_dir,
            metadata_path: default_meta_path,
            score_range: Some(RangeInclusive::new(0, 10)),
            width_range: Some(RangeInclusive::new(0, 10_000)),
            height_range: Some(RangeInclusive::new(0, 10_000)),
        }
    }

    pub fn to_toml_str(&self) -> Result<String> {
        let toml = toml::to_string(&self)?;
        debug!("config serialized to TOML: {}", toml);
        Ok(toml)
    }
}
