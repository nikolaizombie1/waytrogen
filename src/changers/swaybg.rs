use crate::locale::TRANSLATION;
use crate::wallpaper_changers::SwaybgSettings;
use crate::{
    app_state::{AppState, Messages},
    wallpaper_changers::{SwaybgModes, WallpaperChangers},
};
use iced::{
    Element,
    widget::{button, pick_list, text},
};
use iced_aw::helpers::color_picker;
use regex::Regex;
use std::sync::LazyLock;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};
use strum::VariantArray;

#[derive(Default)]
struct SwayBgWallpaper {
    pub settings: SwaybgSettings,
    pub image: PathBuf,
    pub monitor: String,
}

static SWAYBG_RGB_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("#[0-9a-zA-z]{6}").unwrap());

static SWAYBG_WALLPAPERS: LazyLock<Mutex<Vec<SwayBgWallpaper>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub fn change_swaybg_wallpaper(swaybg_changer: WallpaperChangers, image: &Path, monitor: &str) {
    if let WallpaperChangers::Swaybg(settings) = swaybg_changer {
        Command::new("pkill")
            .arg("-9")
            .arg("swaybg")
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        let mut previous_wallpapers = SWAYBG_WALLPAPERS.lock().unwrap();

        if let Some(w) = previous_wallpapers
            .iter_mut()
            .find(|m| m.monitor == monitor)
        {
            w.image.clone_from(&image.to_path_buf());
        } else {
            previous_wallpapers.push(SwayBgWallpaper {
                settings: settings.clone(),
                image: image.to_path_buf(),
                monitor: monitor.to_string(),
            });
        }

        let mut command = Command::new("swaybg");

        if monitor == TRANSLATION.get_translation("All") {
            build_command(&mut command, &settings, image, monitor);
        } else {
            for wallpaper in previous_wallpapers.iter() {
                build_command(
                    &mut command,
                    &wallpaper.settings,
                    &wallpaper.image,
                    &wallpaper.monitor,
                );
            }
        }

        #[allow(clippy::zombie_processes)]
        command.spawn().unwrap();
    }
}

fn build_command(command: &mut Command, settings: &SwaybgSettings, image: &Path, monitor: &str) {
    if monitor != TRANSLATION.get_translation("All") {
        command.arg("-o").arg(monitor);
    }
    let mode = match settings.mode {
        SwaybgModes::Stretch => "stretch",
        SwaybgModes::Fit => "fit",
        SwaybgModes::Fill => "fill",
        SwaybgModes::Center => "center",
        SwaybgModes::Tile => "tile",
        SwaybgModes::SolidColor => "solid_color",
    };
    let fill_color = if SWAYBG_RGB_REGEX.is_match(&settings.fill_color) {
        settings.fill_color.clone()
    } else {
        "#000000".to_string()
    };

    command
        .arg("-i")
        .arg(image.to_str().unwrap())
        .arg("-m")
        .arg(mode)
        .arg("-c")
        .arg(fill_color);
}

pub fn generate_swaybg_changer_bar(app_state: &AppState) -> Vec<Element<'static, Messages>> {
    let dropdown: Element<'_, Messages> = pick_list(
        SwaybgModes::VARIANTS,
        app_state.swaybg_mode.clone(),
        Messages::SwaybgModeChanged,
    )
    .into();
    let color_picker_button = button(text!["{}", TRANSLATION.get_translation("fill-color")])
        .on_press(Messages::ShowSwaybgColorPicker);
    let color_picker_widget: Element<'_, Messages> = color_picker(
        app_state.show_swaybg_color_picker,
        app_state.sway_bg_color_internal,
        color_picker_button,
        Messages::SwaybgFillColorCancelled,
        Messages::SwaybgFillColorSubmitted,
    )
    .into();
    vec![dropdown, color_picker_widget]
}
