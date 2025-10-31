use crate::common::{
    get_config_file_path, parse_executable_script, Wallpaper, APP_ID,
};
use anyhow::anyhow;
use gettextrs::gettext;
use log::{error, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::{remove_file, OpenOptions},
    io::{Read, Write},
};

use gtk::{gio::Settings, prelude::*};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    executable_script_doc: String,
    pub executable_script: String,
    wallpaper_folder_doc: String,
    pub wallpaper_folder: String,
    saved_wallpapers_doc: String,
    pub saved_wallpapers: Vec<Wallpaper>,
    monitor_doc: String,
    pub monitor: u32,
    sort_by_doc: String,
    pub sort_by: u32,
    invert_sort_doc: String,
    pub invert_sort: bool,
    changer_doc: String,
    pub changer: u32,
    image_filter_doc: String,
    pub image_filter: String,
    swaybg_mode_doc: String,
    pub swaybg_mode: u32,
    swaybg_color_doc: String,
    pub swaybg_color: String,
    mpvpaper_pause_option_doc: String,
    pub mpvpaper_pause_option: u32,
    mpvpaper_slideshow_enable_doc: String,
    pub mpvpaper_slideshow_enable: bool,
    mpvpaper_slideshow_interval_doc: String,
    pub mpvpaper_slideshow_interval: f64,
    mpvpaper_additional_options_doc: String,
    pub mpvpaper_additional_options: String,
    selected_monitor_item_doc: String,
    pub selected_monitor_item: String,
    swww_resize_doc: String,
    pub swww_resize: u32,
    swww_fill_color_doc: String,
    pub swww_fill_color: String,
    swww_scaling_filter_doc: String,
    pub swww_scaling_filter: u32,
    swww_transition_type_doc: String,
    pub swww_transition_type: u32,
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
    pub swww_transition_wave_width: f64,
    swww_transition_wave_height_doc: String,
    pub swww_transition_wave_height: f64,
    swww_transition_bezier_p0_doc: String,
    pub swww_transition_bezier_p0: f64,
    swww_transition_bezier_p1_doc: String,
    pub swww_transition_bezier_p1: f64,
    swww_transition_bezier_p2_doc: String,
    pub swww_transition_bezier_p2: f64,
    swww_transition_bezier_p3_doc: String,
    pub swww_transition_bezier_p3: f64,
    swww_transition_fps_doc: String,
    pub swww_transition_fps: u32,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self{
	    executable_script_doc: gettext("The path to executable script used after a wallpaper is set. The script is sent the monitor identifier, wallpaper path and the serialized saved_wallpapers state."),
	    executable_script: String::default(),
	    wallpaper_folder_doc: gettext("The path to the currently selected wallpaper folder. Note: The path cannot have a trailing forward slash."),
	    wallpaper_folder: String::default(),
	    saved_wallpapers_doc: gettext("The collection of the currently saved wallpapers with their corresponding monitor, path and changer."),
	    saved_wallpapers: vec![Wallpaper::default()],
	    monitor_doc: gettext("The internal numeric identifier in the monitor dropdown used by dconf for the currently selected monitor. Do not change unless you know what you are doing."),
	    monitor: u32::default(),
	    sort_by_doc:  gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected sorting option. Do not change unless you know what you are doing."),
	    sort_by: u32::default(),
	    invert_sort_doc: gettext("The boolean flag to invert the currently selected sort-by option in the sort dropdown used by dconf."),
	    invert_sort: bool::default(),
	    changer_doc: gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected changer. Do not change unless you know what you are doing."),
	    changer: u32::default(),
	    image_filter_doc: gettext("The search string for the wallpapers."),
	    image_filter: String::default(),
	    swaybg_mode_doc:  gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected swaybg mode. Do not change unless you know what you are doing."),
	    swaybg_mode: u32::default(),
	    swaybg_color_doc: gettext("The hex color for swaybg background fill. Must be six characters long."),
	    swaybg_color: String::from("000000"),
	    mpvpaper_pause_option_doc: gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected mpvpaper pause option. Do not change unless you know what you are doing."),
	    mpvpaper_pause_option: u32::default(),
	    mpvpaper_slideshow_enable_doc: gettext("The boolean flag to enable/disable slideshows for mpvpaper used by dconf."),
	    mpvpaper_slideshow_enable: bool::default(),
	    mpvpaper_slideshow_interval_doc: gettext("The number of seconds of that mpvpaper takes between switching images in slideshow mode. Note: The option must be a positive floating point number."),
	    mpvpaper_slideshow_interval: f64::default(),
	    mpvpaper_additional_options_doc: gettext("Custom options for mpvpaper passed as command line arguments."),
	    mpvpaper_additional_options: String::default(),
	    selected_monitor_item_doc: gettext("The currently selected monitor as a string. Note: The name must coincide with the monitor numeric identifier."),
	    selected_monitor_item: String::default(),
	    swww_resize_doc: gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww resize option. Do not change unless you know what you are doing."),
	    swww_resize: u32::default(),
	    swww_fill_color_doc:  gettext("The hex color for swww background fill. Must be six characters long."),
	    swww_fill_color: String::from("000000"),
	    swww_scaling_filter_doc: gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww scaling filter option. Do not change unless you know what you are doing."),
	    swww_scaling_filter: u32::default(),
	    swww_transition_type_doc: gettext("The internal numeric identifier in the changer dropdown used by dconf for the currently selected swww transition type option. Do not change unless you know what you are doing."),
	    swww_transition_type: 1,
	    swww_transition_step_doc: gettext("How fast the transition approaches the new image used by swww."),
	    swww_transition_step: 90.0,
	    swww_transition_duration_doc: gettext("How long the transition takes to complete in seconds used by swww."),
	    swww_transition_duration: 3.0,
	    swww_transition_angle_doc: gettext("Used for the 'wipe' and 'wave' transitions used by swww. It controls the angle of the wipe."),
	    swww_transition_angle: 45.0,
	    swww_transition_position_doc: gettext("This is only used for the 'grow','outer' transitions used by swww. It controls the center of circle."),
	    swww_transition_position: String::from("center"),
	    swww_invert_y_doc: gettext("Inverts the y position sent in 'transition_pos' flag used by swww."),
	    swww_invert_y: bool::default(),
	    swww_transition_wave_width_doc: gettext("Currently only used for 'wave' transition to control the width of each wave used by swww."),
	    swww_transition_wave_width: 200.0,
	    swww_transition_wave_height_doc: gettext("Currently only used for 'wave' transition to control the height of each wave used by swww."),
	    swww_transition_wave_height: 200.0,
	    swww_transition_bezier_p0_doc: gettext("Point 0 for the Bezier curve to use for the transition"),
	    swww_transition_bezier_p0: 0.54,
	    swww_transition_bezier_p1_doc: gettext("Point 1 for the Bezier curve to use for the transition"),
	    swww_transition_bezier_p1: 0.0,
	    swww_transition_bezier_p2_doc: gettext("Point 2 for the Bezier curve to use for the transition"),
	    swww_transition_bezier_p2: 0.34,
	    swww_transition_bezier_p3_doc: gettext("Point 3 for the Bezier curve to use for the transition"),
	    swww_transition_bezier_p3: 0.99,
	    swww_transition_fps_doc: gettext("Frame rate for the transition effect used by swww."),
	    swww_transition_fps: 30,
	}
    }
}

impl ConfigFile {
    pub fn write_to_config_file(&self) -> anyhow::Result<()> {
        let config_file = get_config_file_path()?;
        let config_contents = serde_json::to_string_pretty(&self)?;
        let mut config_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&config_file)?;
        config_file.write_all(config_contents.as_bytes())?;
        Ok(())
    }

    pub fn from_gsettings() -> anyhow::Result<Self> {
        let settings = Settings::new(APP_ID);
        let executable_script = get_config_file()?.executable_script;
        trace!("Getting wallpaper-folder gsetting");
        let wallpaper_folder = settings.string("wallpaper-folder").to_string();
        trace!("Getting saved-wallpapers gsetting");
        let saved_wallpapers: Vec<Wallpaper> = serde_json::from_str(
            &settings
                .string("saved-wallpapers")
                .to_string()
                .replace(r"\", ""),
        )?;
        trace!("Getting monitor gsetting");
        let monitor = settings.uint("monitor");
        trace!("Getting sort-by gsetting");
        let sort_by = settings.uint("sort-by");
        trace!("Getting invert-sort gsetting");
        let invert_sort = settings.boolean("invert-sort");
        trace!("Getting changer gsetting");
        let changer = settings.uint("changer");
        trace!("Getting image-filter gsetting");
        let image_filter = settings.string("image-filter").to_string();
        trace!("Getting swaybg-mode gsetting");
        let swaybg_mode = settings.uint("swaybg-mode");
        trace!("Getting swaybg-color gsetting");
        let swaybg_color = settings.string("swaybg-color").to_string();
        trace!("Getting mpvpaper-pause-option gsetting");
        let mpvpaper_pause_option = settings.uint("mpvpaper-pause-option");
        trace!("Getting mpvpaper-slideshow-enable gsetting");
        let mpvpaper_slideshow_enable = settings.boolean("mpvpaper-slideshow-enable");
        trace!("Getting mpvpaper-slideshow-interval gsetting");
        let mpvpaper_slideshow_interval = settings.double("mpvpaper-slideshow-interval");
        trace!("Getting mpvpaper-additional-options gsetting");
        let mpvpaper_additional_options =
            settings.string("mpvpaper-additional-options").to_string();
        trace!("Getting selected-monitor-item gsetting");
        let selected_monitor_item = settings.string("selected-monitor-item").to_string();
        trace!("Getting swww-resize gsetting");
        let swww_resize = settings.uint("swww-resize");
        trace!("Getting swww-fill-color gsetting");
        let swww_fill_color = settings.string("swww-fill-color").to_string();
        trace!("Getting swww-scaling-filter gsetting");
        let swww_scaling_filter = settings.uint("swww-scaling-filter");
        trace!("Getting swww-transition-type gsetting");
        let swww_transition_type = settings.uint("swww-transition-type");
        trace!("Getting swww-transition-step gsetting");
        let swww_transition_step = settings.double("swww-transition-step");
        trace!("Getting swww-transition-duration gsetting");
        let swww_transition_duration = settings.double("swww-transition-duration");
        trace!("Getting swww-transition-angle gsetting");
        let swww_transition_angle = settings.double("swww-transition-angle");
        trace!("Getting swww-transition-position gsetting");
        let swww_transition_position = settings.string("swww-transition-position").to_string();
        trace!("Getting swww-invert-y gsetting");
        let swww_invert_y = settings.boolean("swww-invert-y");
        trace!("Getting swww-transition-wave-width gsetting");
        let swww_transition_wave_width = settings.double("swww-transition-wave-width");
        trace!("Getting swww-transition-wave-height gsetting");
        let swww_transition_wave_height = settings.double("swww-transition-wave-height");
        trace!("Getting swww-transition-bezier-p0 gsetting");
        let swww_transition_bezier_p0 = settings.double("swww-transition-bezier-p0");
        trace!("Getting swww-transition-bezier-p1 gsetting");
        let swww_transition_bezier_p1 = settings.double("swww-transition-bezier-p1");
        trace!("Getting swww-transition-bezier-p2 gsetting");
        let swww_transition_bezier_p2 = settings.double("swww-transition-bezier-p2");
        trace!("Getting swww-transition-bezier-p3 gsetting");
        let swww_transition_bezier_p3 = settings.double("swww-transition-bezier-p3");
        trace!("Getting swww-transition-fps gsetting");
        let swww_transition_fps = settings.uint("swww-transition-fps");

        Ok(Self {
            executable_script,
            wallpaper_folder,
            saved_wallpapers,
            monitor,
            sort_by,
            invert_sort,
            changer,
            image_filter,
            swaybg_mode,
            swaybg_color,
            mpvpaper_pause_option,
            mpvpaper_slideshow_enable,
            mpvpaper_slideshow_interval,
            mpvpaper_additional_options,
            selected_monitor_item,
            swww_resize,
            swww_fill_color,
            swww_scaling_filter,
            swww_transition_type,
            swww_transition_step,
            swww_transition_duration,
            swww_transition_angle,
            swww_transition_position,
            swww_invert_y,
            swww_transition_wave_width,
            swww_transition_wave_height,
            swww_transition_bezier_p0,
            swww_transition_bezier_p1,
            swww_transition_bezier_p2,
            swww_transition_bezier_p3,
            swww_transition_fps,
            ..Default::default()
        })
    }

    pub fn write_to_gsettings(&self) -> anyhow::Result<()> {
        let settings = Settings::new(APP_ID);
        settings.set_string("wallpaper-folder", &self.wallpaper_folder)?;
        settings.set_string(
            "saved-wallpapers",
            &serde_json::to_string_pretty(&self.saved_wallpapers)?,
        )?;
        trace!("Setting monitor gsetting.");
        settings.set_uint("monitor", self.monitor)?;
        trace!("Setting sort-by gsetting");
        settings.set_uint("sort-by", self.sort_by)?;
        trace!("Setting invert-sort gsetting");
        settings.set_boolean("invert-sort", self.invert_sort)?;
        trace!("Setting changer gsetting");
        settings.set_uint("changer", self.changer)?;
        trace!("Setting image-filter gsetting");
        settings.set_string("image-filter", &self.image_filter)?;
        trace!("Setting swaybg-mode gsetting");
        settings.set_uint("swaybg-mode", self.swaybg_mode)?;
        trace!("Setting swaybg-color gsetting");
        settings.set_string("swaybg-color", &self.swaybg_color)?;
        trace!("Setting mpvpaper-pause-option gsetting");
        settings.set_uint("mpvpaper-pause-option", self.mpvpaper_pause_option)?;
        trace!("Setting mpvpaper-slideshow-enable gsetting");
        settings.set_boolean("mpvpaper-slideshow-enable", self.mpvpaper_slideshow_enable)?;
        trace!("Setting mpvpaper-slideshow-interval gsetting");
        settings.set_double(
            "mpvpaper-slideshow-interval",
            self.mpvpaper_slideshow_interval,
        )?;
        trace!("Setting mpvpaper-additional-options gsetting");
        settings.set_string(
            "mpvpaper-additional-options",
            &self.mpvpaper_additional_options,
        )?;
        trace!("Setting selected-monitor-item gsetting");
        settings.set_string("selected-monitor-item", &self.selected_monitor_item)?;
        trace!("Setting swww-resize gsetting");
        settings.set_uint("swww-resize", self.swww_resize)?;
        trace!("Setting swww-fill-color gsetting");
        settings.set_string("swww-fill-color", &self.swww_fill_color)?;
        trace!("Setting swww-scaling-filter gsetting");
        settings.set_uint("swww-scaling-filter", self.swww_scaling_filter)?;
        trace!("Setting swww-transition-type gsetting");
        settings.set_uint("swww-transition-type", self.swww_transition_type)?;
        trace!("Setting swww-transition-step gsetting");
        settings.set_double("swww-transition-step", self.swww_transition_step)?;
        trace!("Setting swww-transition-duration gsetting");
        settings.set_double("swww-transition-duration", self.swww_transition_duration)?;
        trace!("Setting swww-transition-angle gsetting");
        settings.set_double("swww-transition-angle", self.swww_transition_angle)?;
        trace!("Setting swww-transition-position gsetting");
        settings.set_string("swww-transition-position", &self.swww_transition_position)?;
        trace!("Setting swww-invert-y gsetting");
        settings.set_boolean("swww-invert-y", self.swww_invert_y)?;
        trace!("Setting swww-transition-wave-width gsetting");
        settings.set_double(
            "swww-transition-wave-width",
            self.swww_transition_wave_width,
        )?;
        trace!("Setting swww-transition-wave-height gsetting");
        settings.set_double(
            "swww-transition-wave-height",
            self.swww_transition_wave_height,
        )?;
        trace!("Setting swww-transition-bezier-p0 gsetting");
        settings.set_double("swww-transition-bezier-p0", self.swww_transition_bezier_p0)?;
        trace!("Setting swww-transition-bezier-p1 gsetting");
        settings.set_double("swww-transition-bezier-p1", self.swww_transition_bezier_p1)?;
        trace!("Setting swww-transition-bezier-p2 gsetting");
        settings.set_double("swww-transition-bezier-p2", self.swww_transition_bezier_p2)?;
        trace!("Setting swww-transition-bezier-p3 gsetting");
        settings.set_double("swww-transition-bezier-p3", self.swww_transition_bezier_p3)?;
        trace!("Setting swww-transition-fps gsetting");
        settings.set_uint("swww-transition-fps", self.swww_transition_fps)?;
        Ok(())
    }
}

pub fn get_config_file() -> anyhow::Result<ConfigFile> {
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

    let config_file_struct = match serde_json::from_str::<ConfigFile>(&config_contents) {
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
            let config_file = ConfigFile::default();
            let config_string = serde_json::to_string_pretty::<ConfigFile>(&config_file)?;
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
