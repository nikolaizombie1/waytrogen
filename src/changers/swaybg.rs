use crate::{
    app_state::{AppState, Messages},
    common::DEFAULT_MARGIN,
    wallpaper_changers::{SwaybgModes, WallpaperChangers},
};
use gettextrs::gettext;
use iced::{
    Element,
    widget::{button, pick_list, row, text},
};
use iced_aw::helpers::color_picker;
use std::{path::Path, process::Command};
use strum::VariantArray;

pub fn change_swaybg_wallpaper(swaybg_changer: WallpaperChangers, image: &Path, monitor: &str) {
    if let WallpaperChangers::Swaybg(settings) = swaybg_changer {
        let mut command = Command::new("swaybg");
        if monitor != gettext("All") {
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
        command
            .arg("-i")
            .arg(image.to_str().unwrap())
            .arg("-m")
            .arg(mode)
            .arg("-c")
            .arg(settings.fill_color)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_swaybg_changer_bar(app_state: &AppState) -> Element<'_, Messages> {
    let dropdown = pick_list(
        SwaybgModes::VARIANTS,
        app_state.swaybg_mode.clone(),
        Messages::SwaybgModeChanged,
    );
    let color_picker_button =
        button(text!["{}", gettext("Fill Color")]).on_press(Messages::ShowSwaybgColorPicker);
    let color_picker_widget = color_picker(
        app_state.show_swaybg_color_picker,
        app_state.sway_bg_color_internal,
        color_picker_button,
        Messages::SwaybgFillColorCancelled,
        Messages::SwaybgFillColorSubmitted,
    );
    row![dropdown, color_picker_widget]
        .spacing(DEFAULT_MARGIN as f32)
        .into()
}
