use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::utils;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ColorTheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Color {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "darkgray")]
    DarkGray,
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "white")]
    White,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "pink")]
    Pink,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageMeta {
    // blake3 hash
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub scores: Vec<ImageScore>,
    pub tags: Vec<String>,
    pub theme: Option<ColorTheme>,
    pub colors: Vec<Color>,
}

impl ImageMeta {
    pub fn create_from_image(image: &PathBuf) -> Result<ImageMeta> {
        let id = utils::common::compute_blake3_hash(image)?;
        let filename = image
            .file_name()
            .context("image file should have a filename")?
            .to_string_lossy()
            .into_owned();

        let dimensions = utils::common::get_image_dims(image)?;

        let meta = ImageMeta {
            id,
            path: image.to_path_buf(),
            title: filename.to_owned(),
            description: String::from(""),
            width: dimensions.0,
            height: dimensions.1,
            scores: vec![],
            tags: vec![],
            theme: None,
            colors: vec![],
        };

        Ok(meta)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageScore {
    pub name: String,
    pub value: u8,
}
