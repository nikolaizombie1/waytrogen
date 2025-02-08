use crate::{common::RGB, wallpaper_changers::WallpaperChangers};
use gettextrs::gettext;
use gtk::{
    gdk::RGBA,
    gio::Settings,
    glib::{self, clone},
    prelude::*,
    Align, Box, ColorDialog, ColorDialogButton, DropDown, TextBuffer,
};
use log::debug;
use std::{path::Path, process::Command};

pub fn change_swaybg_wallpaper(swaybg_changer: WallpaperChangers, image: &Path, monitor: &str) {
    if let WallpaperChangers::Swaybg(mode, rgb) = swaybg_changer {
        let mut command = Command::new("swaybg");
        if monitor != gettext("All") {
            command.arg("-o").arg(monitor);
        }
        command
            .arg("-i")
            .arg(image.to_str().unwrap())
            .arg("-m")
            .arg(mode.to_string())
            .arg("-c")
            .arg(rgb)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_swaybg_changer_bar(changer_specific_options_box: &Box, settings: &Settings) {
    let dropdown = DropDown::from_strings(&[
        &gettext("stretch"),
        &gettext("fit"),
        &gettext("fill"),
        &gettext("center"),
        &gettext("tile"),
        &gettext("solid_color"),
    ]);
    dropdown.set_halign(Align::Start);
    dropdown.set_valign(Align::Center);
    dropdown.set_margin_top(12);
    dropdown.set_margin_start(12);
    dropdown.set_margin_bottom(12);
    dropdown.set_margin_end(12);
    changer_specific_options_box.append(&dropdown);
    let color_dialog = ColorDialog::builder().with_alpha(false).build();
    let color_picker = ColorDialogButton::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .dialog(&color_dialog)
        .build();
    changer_specific_options_box.append(&color_picker);
    settings.bind("swaybg-mode", &dropdown, "selected").build();
    let rgb_text_buffer = TextBuffer::builder().build();
    color_picker.connect_rgba_notify(clone!(
        #[weak]
        settings,
        move |b| {
            let rgba = b.rgba();
            let serialize_struct = RGB {
                red: rgba.red(),
                green: rgba.green(),
                blue: rgba.blue(),
            }
            .to_string();
            debug!("{}: {}", gettext("Serialized RGB"), serialize_struct);
            rgb_text_buffer.set_text(&serialize_struct);
            settings
                .bind("swaybg-color", &rgb_text_buffer, "text")
                .build();
        }
    ));
    let rgb = settings
        .string("swaybg-color")
        .to_string()
        .parse::<RGB>()
        .unwrap();
    color_picker.set_rgba(
        &RGBA::builder()
            .red(rgb.red)
            .green(rgb.green)
            .blue(rgb.blue)
            .build(),
    );
}
