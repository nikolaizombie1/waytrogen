use crate::common::{parse_executable_script, CONFIG_APP_NAME, CONFIG_FILE_NAME};
use anyhow::anyhow;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    pub executable_script: String,
}

pub fn get_config_file() -> anyhow::Result<ConfigFile> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME)?;
    let config_file = xdg_dirs.place_config_file(CONFIG_FILE_NAME)?;
    let mut config = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(&config_file);
    if config.is_err() {
        warn!(
            "Config file '{}' was not found: Attempting to create a new one.",
            config_file.to_str().unwrap_or_default()
        );
        config = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&config_file);
    }
    let mut config = config?;
    let mut config_contents = String::new();
    let _ = config.read_to_string(&mut config_contents)?;
    let mut config_file_struct = ConfigFile::default();
    if config_contents.is_empty() {
        let config_file_struct = ConfigFile::default();
        let config_string = serde_json::to_string_pretty::<ConfigFile>(&config_file_struct)?;
        let bytes_written = config.write(config_string.as_bytes())?;
        if bytes_written != config_string.len() {
            error!("Failed to write config file");
            return Err(anyhow!("Failed to write config file"));
        }
    } else {
        config_file_struct = serde_json::from_str::<ConfigFile>(&config_contents)?;
    }
    let _ = parse_executable_script(&config_file_struct.executable_script)?;
    Ok(config_file_struct)
}
