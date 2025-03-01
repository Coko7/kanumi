use std::path::PathBuf;

use serde::{Deserialize, Serialize};

type UUID = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum ColorTheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageMeta {
    pub id: UUID,
    pub path: PathBuf,
    pub title: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub scores: Vec<ImageScore>,
    pub tags: Vec<String>,
    pub theme: ColorTheme,
    pub colors: Vec<Color>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageScore {
    pub name: String,
    pub value: u8,
}
