use crate::locale::TRANSLATION;
use crate::{
    app_state::{AppState, Messages},
    wallpaper_changers::{SwaybgModes, WallpaperChangers},
};
use iced::{
    Element,
    widget::{button, pick_list, text},
};
use iced_aw::helpers::color_picker;
use std::{path::Path, process::Command};
use strum::VariantArray;

pub fn change_swaybg_wallpaper(swaybg_changer: WallpaperChangers, image: &Path, monitor: &str) {
    if let WallpaperChangers::Swaybg(settings) = swaybg_changer {
        let mut command = Command::new("swaybg");
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
	let fill_color = if settings.fill_color.is_empty() {
	    "#000000".to_string()
	} else {
	    settings.fill_color
	};
        command
            .arg("-i")
            .arg(image.to_str().unwrap())
            .arg("-m")
            .arg(mode)
            .arg("-c")
            .arg(fill_color)
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
    }
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
