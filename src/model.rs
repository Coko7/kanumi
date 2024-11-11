use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
    pub root_images_dir: PathBuf,

    #[serde(rename = "meta_path")]
    pub metadata_path: Option<PathBuf>,

    #[serde(rename = "score")]
    pub score_range: Option<String>,
}
