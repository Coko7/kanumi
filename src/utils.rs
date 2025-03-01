use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::{env, fs, ops::RangeInclusive, path::PathBuf, usize};
use xdg::BaseDirectories;

use crate::{cli::ScoreFilter, config::Configuration, image_meta::ImageMeta};

pub const APP_NAME: &str = "kanumi";
pub const CONFIG_VAR: &str = "KANUMI_CONFIG";

pub fn get_config_dir() -> Result<PathBuf, anyhow::Error> {
    if let Ok(config_var) = env::var(CONFIG_VAR) {
        let val = PathBuf::from(config_var);
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

pub fn parse_score_filters(input: &str) -> Result<ScoreFilter, anyhow::Error> {
    let mut allow_unscored = false;
    let mut input = input.to_string();

    if input.ends_with('@') {
        input = input
            .strip_suffix('@')
            .context("failed to strip ! suffix on score filter")?
            .to_string();

        allow_unscored = true;
    }

    let mut parts = input.split('=');
    let key = parts.next().context("failed to get key")?.to_string();
    let range = parts.next().context("failed to get range")?.to_string();
    let range = parse_range(&range)?;

    let score_filter = ScoreFilter {
        name: key,
        range,
        allow_unscored,
    };

    Ok(score_filter)
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
        [Some(start), Some(end)] => {
            if start > end {
                return Err(anyhow!("start should be <= end: {} > {}", start, end));
            }
            Ok(*start..=*end)
        }
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

pub fn image_score_matches(meta: &ImageMeta, score_filter: &ScoreFilter) -> bool {
    let img_score = meta
        .scores
        .iter()
        .find(|score| score.name == score_filter.name);

    if let Some(img_score) = img_score {
        return usize::from(img_score.value) >= *score_filter.range.start()
            && usize::from(img_score.value) <= *score_filter.range.end();
    }

    score_filter.allow_unscored
}

pub fn load_image_metas(meta_file_path: PathBuf) -> Result<Vec<ImageMeta>> {
    let data = fs::read_to_string(meta_file_path)?;
    let metas: Vec<ImageMeta> = serde_json::from_str(&data)?;
    Ok(metas)
}
