use crate::common::{parse_executable_script, Wallpaper, CONFIG_APP_NAME, CONFIG_FILE_NAME};
use anyhow::anyhow;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    executable_script_doc: String,
    pub executable_script: String,
    wallpaper_folder_doc: String,
    pub wallpaper_folder: String,
    saved_wallpapers_doc: String,
    pub saved_wallpapers: Vec<Wallpaper>,
    monitor_doc: String, 
    pub monitor: usize, 
    sort_by_doc: String,
    pub sort_by: usize,
    invert_sort_doc: String,
    pub invert_sort: bool,
    changer_doc: String,
    pub changer: usize,
    image_filter_doc: String,
    pub image_filter: String,
    swaybg_mode_doc: String,
    pub swaybg_mode: usize,
    swaybg_color_doc: String,
    pub swaybg_color: String,
    mpvpaper_pause_option_doc: String,
    pub mpvpaper_pause_option: usize,
    mpvpaper_slideshow_enable_doc: String,
    pub mpvpaper_slideshow_enable: bool,
    mpvpaper_slideshow_interval_doc: String,
    pub mpvpaper_slideshow_interval: f64,
    mpvpaper_additional_options_doc: String,
    pub mpvpaper_additional_options: String,
    selected_monitor_item_doc: String,
    pub selected_monitor_item: String,
    swww_resize_doc: String,
    pub swww_resize: usize,
    swww_fill_color_doc: String,
    pub swww_fill_color: String,
    swww_scaling_filter_doc: String,
    pub swww_scaling_filter: usize,
    swww_transition_type_doc: String,
    pub swww_transition_type: usize,
    swww_transition_step_doc: String,
    pub swww_transition_step: f64,
    swww_transition_duration_doc: String,
    pub swww_transition_duration: f64,
    swww_transition_angle_doc: String,
    pub swww_transition_angle: f64,
    swww_transition_position_doc: String,
    pub swww_transition_position: String,
    swww_invert_y_doc: String,
    pub swww_invert_y: bool,
    swww_transition_wave_width_doc: String,
    pub swww_transition_wave_width: usize,
    swww_transition_wave_height_doc: String,
    pub swww_transition_wave_height: usize,
    swww_transition_bezier_p0_doc: String,
    pub swww_transition_bezier_p0: f64,
    swww_transition_bezier_p1_doc: String,
    pub swww_transition_bezier_p1: f64,
    swww_transition_bezier_p2_doc: String,
    pub swww_transition_bezier_p2: f64,
    swww_transition_bezier_p3_doc: String,
    pub swww_transition_bezier_p3: f64,
    swww_transition_fps_doc: String,
    pub swww_transition_fps: usize,
}

impl Default for ConfigFile {
    fn default() -> Self {
	Self{
	    executable_script_doc: "The path to executable script used after a wallpaper is set. The script is sent the monitor identifier, wallpaper path and the serialized saved_wallpapers state.".to_owned(),
	    executable_script: String::default(),
	    wallpaper_folder_doc: "The path to the currently selected wallpaper folder. Note: The path cannot have a trailing forward slash.".to_owned(),
	    wallpaper_folder: String::default(),
	    saved_wallpapers_doc: "The collection of the currently saved wallpapers with their corresponding monitor, path and changer.".to_owned(),
	    saved_wallpapers: vec![Wallpaper::default()],
	    monitor_doc: "The internal numeric identifier in the monitor dropdown used by dconf for the currently selected monitor. Do not change unless you know what you are doing.".to_owned(),
	    monitor: usize::default(),
	    sort_by_doc:  "The internal numeric identifier in the changer dropdown used by dconf for the currently selected sorting option. Do not change unless you know what you are doing.".to_owned(),
	    sort_by: usize::default(),
	    invert_sort_doc: "The boolean flag to invert the currently selected sort-by option in the sort dropdown used by dconf.".to_owned(),
	    invert_sort: bool::default(),
	    changer_doc: "The internal numeric identifier in the changer dropdown used by dconf for the currently selected changer. Do not change unless you know what you are doing.".to_owned(),
	    changer: usize::default(),
	    image_filter_doc: "The search string for the wallpapers.".to_owned(),
	    image_filter: String::default(),
	    swaybg_mode_doc:  "The internal numeric identifier in the changer dropdown used by dconf for the currently selected swaybg mode. Do not change unless you know what you are doing.".to_owned(),
	    swaybg_mode: usize::default(),
	    swaybg_color_doc: "The hex color for swaybg background fill. Must be six characters long.".to_owned(),
	    swaybg_color: String::from("000000"),
	    mpvpaper_pause_option_doc: "The internal numeric identifier in the changer dropdown used by dconf for the currently selected mpvpaper pause option. Do not change unless you know what you are doing.".to_owned(),
	    mpvpaper_pause_option: usize::default(),
	    mpvpaper_slideshow_enable_doc: "The boolean flag to enable/disable slideshows for mpvpaper used by dconf.".to_owned(),
	    mpvpaper_slideshow_enable: bool::default(),
	    mpvpaper_slideshow_interval_doc: "The number of seconds of that mpvpaper takes between switching images in slideshow mode. Note: The option must be a positive floating point number.".to_owned(),
	    mpvpaper_slideshow_interval: f64::default(),
	    mpvpaper_additional_options_doc: "Custom options for mpvpaper passed as command line arguments.".to_owned(),
	    mpvpaper_additional_options: String::default(),
	    selected_monitor_item_doc: "The currently selected monitor as a string. Note: The name must coincide with the monitor numeric identifier.".to_owned(),
	    selected_monitor_item: String::default(),
	    swww_resize_doc: "The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww resize option. Do not change unless you know what you are doing.".to_owned(),
	    swww_resize: usize::default(),
	    swww_fill_color_doc:  "The hex color for swww background fill. Must be six characters long.".to_owned(),
	    swww_fill_color: String::from("000000"),
	    swww_scaling_filter_doc: "The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww scaling filter option. Do not change unless you know what you are doing.".to_owned(),
	    swww_scaling_filter: usize::default(),
	    swww_transition_type_doc: "The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww transition type option. Do not change unless you know what you are doing.".to_owned(),
	    swww_transition_type: 1,
	    swww_transition_step_doc: "How fast the transition approaches the new image used by swww.".to_owned(),
	    swww_transition_step: 90.0,
	    swww_transition_duration_doc: "How long the transition takes to complete in seconds used by swww.".to_owned(),
	    swww_transition_duration: 3.0,
	    swww_transition_angle_doc: "Used for the 'wipe' and 'wave' transitions used by swww. It controls the angle of the wipe.".to_owned(),
	    swww_transition_angle: 45.0,
	    swww_transition_position_doc: "This is only used for the 'grow','outer' transitions used by swww. It controls the center of circle.".to_owned(),
	    swww_transition_position: String::from("center"),
	    swww_invert_y_doc: "Inverts the y position sent in 'transition_pos' flag used by swww.".to_owned(),
	    swww_invert_y: bool::default(),
	    swww_transition_wave_width_doc: "Currently only used for 'wave' transition to control the width of each wave used by swww.".to_owned(),
	    swww_transition_wave_width: 200,
	    swww_transition_wave_height_doc: "Currently only used for 'wave' transition to control the height of each wave used by swww.".to_owned(),
	    swww_transition_wave_height: 200,
	    swww_transition_bezier_p0_doc: "Point 0 for the Bezier curve to use for the transition".to_owned(),
	    swww_transition_bezier_p0: 0.54,
	    swww_transition_bezier_p1_doc: "Point 1 for the Bezier curve to use for the transition".to_owned(),
	    swww_transition_bezier_p1: 0.0,
	    swww_transition_bezier_p2_doc: "Point 2 for the Bezier curve to use for the transition".to_owned(),
	    swww_transition_bezier_p2: 0.34,
	    swww_transition_bezier_p3_doc: "Point 3 for the Bezier curve to use for the transition".to_owned(),
	    swww_transition_bezier_p3: 0.99,
	    swww_transition_fps_doc: "Frame rate for the transition effect used by swww.".to_owned(),
	    swww_transition_fps: 30,
	    
	}
    }
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
