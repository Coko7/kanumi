use std::{env, fs, path::PathBuf};

use anyhow::anyhow;
use log::{debug, error, info};
use xdg::BaseDirectories;

use crate::model::Configuration;

pub fn get_config_dir() -> Result<PathBuf, anyhow::Error> {
    let app_name = "kanumi";
    let config_var = "KANUMI_CONFIG";

    if env::var(config_var).is_ok() {
        let val = PathBuf::from(config_var);
        info!(
            "get config from env: {} = {}",
            config_var,
            val.to_string_lossy()
        );

        return Ok(val);
    }

    if let Ok(xdg_dirs) = BaseDirectories::new() {
        let config_home = xdg_dirs.get_config_home();
        let val = config_home.join(app_name);
        info!("get config from XDG: {}", val.to_string_lossy());

        return Ok(val);
    }

    if let Ok(home_dir) = env::var("HOME") {
        let val = PathBuf::from(home_dir).join(".config").join(app_name);
        info!("get config from HOME: {}", val.to_string_lossy());

        return Ok(val);
    }

    error!("No suitable place for config dir");
    Err(anyhow!("Failed to find config dir"))
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

    let default_root_dir = PathBuf::from("/home/coco/Pictures/Wallpapers/");

    let default_config = Configuration {
        root_images_dir: default_root_dir.clone(),
        metadata_path: Some(default_root_dir.join("metadata.csv")),
        score_range: Some("0..".to_string()),
    };

    let toml = toml::to_string(&default_config).unwrap();
    debug!("default config serialized TOML: {}", toml);

    fs::write(&file_path, toml)?;

    Ok(())
}

pub fn load_config(path: PathBuf) -> Result<Configuration, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    info!("parsing config toml");
    let config: Configuration = toml::from_str(&content)?;

    Ok(config)
}
