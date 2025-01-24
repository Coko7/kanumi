use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::{
    env,
    fs::{self, File},
    ops::RangeInclusive,
    path::PathBuf,
    usize,
};
use xdg::BaseDirectories;

use crate::model::{Configuration, ImageMeta};

pub const APP_NAME: &str = "kanumi";
pub const CONFIG_VAR: &str = "KANUMI_CONFIG";

pub fn get_config_dir() -> Result<PathBuf, anyhow::Error> {
    if env::var(CONFIG_VAR).is_ok() {
        let val = PathBuf::from(CONFIG_VAR);
        info!(
            "get config from env: {} = {}",
            CONFIG_VAR,
            val.to_string_lossy()
        );

        return Ok(val);
    }

    if let Ok(xdg_dirs) = BaseDirectories::new() {
        let config_home = xdg_dirs.get_config_home();
        let val = config_home.join(APP_NAME);
        info!("get config from XDG: {}", val.to_string_lossy());

        return Ok(val);
    }

    if let Ok(home_dir) = env::var("HOME") {
        let val = PathBuf::from(home_dir).join(".config").join(APP_NAME);
        info!("get config from HOME: {}", val.to_string_lossy());

        return Ok(val);
    }

    Err(anyhow!("could not get config directory"))
}

pub fn get_config_file() -> Result<PathBuf, anyhow::Error> {
    Ok(get_config_dir()?.join("config.toml"))
}

pub fn create_config_file() -> Result<(), anyhow::Error> {
    let file_path = get_config_file()?;
    info!("create config file: `{}`", file_path.to_string_lossy());

    if let Some(config_dir) = file_path.parent() {
        fs::create_dir_all(config_dir)?;
    }

    let default_config = Configuration::create_default();
    let toml = default_config.to_toml_str()?;

    fs::write(&file_path, toml)?;
    Ok(())
}

pub fn load_config(path: PathBuf) -> Result<Configuration, anyhow::Error> {
    let content = fs::read_to_string(path)?;

    info!("parsing config toml");
    let config: Configuration = toml::from_str(&content)?;
    Ok(config)
}

pub fn parse_range(input: &str) -> Result<RangeInclusive<usize>, anyhow::Error> {
    if let Ok(num) = input.parse::<usize>() {
        return Ok(num..=num);
    }

    if !input.contains("..") {
        return Err(anyhow!(
            "expected number N or range (N..O) but got: `{}`",
            input
        ));
    }

    let parts: Vec<&str> = input.split("..").collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "invalid range format, expected X..Y but got: `{}`",
            input
        ));
    }

    let mut formatted_parts = Vec::new();
    for part in parts {
        if part.is_empty() {
            formatted_parts.push(None);
        } else {
            match part.parse::<usize>() {
                Ok(num) => formatted_parts.push(Some(num)),
                Err(e) => return Err(anyhow!("failed to parse number: `{}`", e)),
            }
        }
    }

    match formatted_parts.as_slice() {
        [None, Some(end)] => Ok(0..=*end),
        [Some(start), None] => Ok(*start..=usize::MAX),
        [Some(start), Some(end)] => Ok(*start..=*end),
        _ => Err(anyhow!("range should have at least one boundary")),
    }
}

pub fn image_matches_dims(
    image: &PathBuf,
    width_range: &Option<RangeInclusive<usize>>,
    height_range: &Option<RangeInclusive<usize>>,
) -> bool {
    debug!("checking dimensions for: {}", image.display());
    let dimensions = image::image_dimensions(image);
    if dimensions.is_err() {
        warn!(
            "failed to check dimensions for: {}, error: {}",
            image.display(),
            dimensions.unwrap_err()
        );
        return false;
    }

    let dimensions = dimensions.unwrap();
    let (width, height) = (dimensions.0 as usize, dimensions.1 as usize);

    if let Some(width_range) = width_range {
        if !width_range.contains(&width) {
            return false;
        }
    }

    if let Some(height_range) = height_range {
        if !height_range.contains(&height) {
            return false;
        }
    }

    true
}

pub fn image_score_matches(meta: &ImageMeta, range: &RangeInclusive<usize>) -> bool {
    usize::from(meta.score) >= *range.start() && usize::from(meta.score) <= *range.end()
}

pub fn create_meta_list(meta_file_path: PathBuf) -> Result<Vec<ImageMeta>> {
    let mut reader = csv::Reader::from_reader(File::open(meta_file_path)?);

    let mut records = Vec::new();
    for entry in reader.deserialize() {
        let record: ImageMeta = entry?;
        records.push(record);
    }

    Ok(records)
}
