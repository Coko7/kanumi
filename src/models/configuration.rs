use anyhow::Result;
use directories::UserDirs;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, path::PathBuf};

use super::ScoreFilter;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(rename = "root_path")]
    pub root_images_dir: PathBuf,

    #[serde(rename = "meta_path")]
    pub metadata_path: PathBuf,

    #[serde(rename = "filters")]
    pub filters: ConfigurationFilters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationFilters {
    #[serde(rename = "active_dirs")]
    pub active_directories: Option<Vec<PathBuf>>,

    #[serde(rename = "scores")]
    pub scores: Option<Vec<ScoreFilter>>,

    #[serde(rename = "width")]
    pub width_range: Option<RangeInclusive<usize>>,

    #[serde(rename = "height")]
    pub height_range: Option<RangeInclusive<usize>>,
}

impl Configuration {
    pub fn create_default() -> Configuration {
        let mut root_images_dir = PathBuf::new();
        let mut metadata_path = PathBuf::new();

        if let Some(user_dirs) = UserDirs::new() {
            info!("found user dirs: {:?}", user_dirs);
            if let Some(picture_dir) = user_dirs.picture_dir() {
                info!("found user pictures dir: {}", picture_dir.display());
                root_images_dir = picture_dir.to_path_buf();
                metadata_path = picture_dir.join("metadatas.json");
            }
        }

        let filters = ConfigurationFilters {
            active_directories: None,
            scores: None,
            width_range: Some(RangeInclusive::new(0, 10_000)),
            height_range: Some(RangeInclusive::new(0, 10_000)),
        };

        Configuration {
            root_images_dir,
            metadata_path,
            filters,
        }
    }

    pub fn to_toml_str(&self) -> Result<String> {
        let toml = toml::to_string(&self)?;
        debug!("config serialized to TOML: {}", toml);
        Ok(toml)
    }
}
