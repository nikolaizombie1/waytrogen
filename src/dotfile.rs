use crate::{
    app_state::AppState,
    common::{get_config_file_path, parse_executable_script},
};
use anyhow::anyhow;
use log::{error, trace, warn};
use std::{
    fs::{OpenOptions, remove_file},
    io::{Read, Write},
};

pub fn get_config_file() -> anyhow::Result<AppState> {
    let config_file = get_config_file_path()?;
    let mut config = match config_file.exists() {
        true => OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&config_file)?,
        false => {
            warn!("Config file was not found: Attempting to create a new one.");
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&config_file)?
        }
    };
    let mut config_contents = String::new();
    let _ = config.read_to_string(&mut config_contents)?;

    let config_file_struct = match serde_json::from_str::<AppState>(&config_contents) {
        Ok(s) => {
            trace!("{}", "Successfully obtained configuration file");
            s
        }
        Err(_) => {
            remove_file(&config_file)?;
            config = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&config_file)?;
            let config_file = AppState::default();
            let config_string = serde_json::to_string_pretty::<AppState>(&config_file)?;
            config.write_all(config_string.as_bytes())?;
            config_file
        }
    };

    match parse_executable_script(&config_file_struct.executable_script) {
        Ok(_) => {
            trace!("{}", "Successfully parsed executable script");
        }
        Err(e) => {
            error!("Failed to parse executable script: {e}");
            return Err(anyhow!("Failed to parse executable script: {e}"));
        }
    };
    Ok(config_file_struct)
}
