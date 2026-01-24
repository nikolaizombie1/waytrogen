use gettextrs::gettext;
use gtk::{Align, Box, DropDown, gio::Settings, prelude::*};
use log::{debug, error, warn};
use std::{path::Path, process::Command, thread, time::Duration};
use which::which;

use crate::{ui_common::DEFAULT_MARGIN, wallpaper_changers::WallpaperChangers};

pub fn change_hyprpaper_wallpaper(
    hyprpaper_changer: WallpaperChangers,
    image: &Path,
    monitor: &str,
) {
    if let WallpaperChangers::Hyprpaper(fit_mode) = hyprpaper_changer {
        debug!("Starting hyprpaper");
        if !Command::new("pgrep")
            .arg("hyprpaper")
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success()
        {
            match Command::new("systemctl")
                .arg("--user")
                .arg("start")
                .arg("hyprpaper")
                .spawn()
                .unwrap()
                .wait()
            {
                Ok(_) => {}
                Err(_) => match which("hyprpaper") {
                    Ok(_) => {
                        warn!("Hyprpaper could not be started using Systemd. Attempting to start using command line interface");
                        #[allow(clippy::zombie_processes)]
                        Command::new("hyprpaper").spawn().unwrap();
                    }
                    Err(_) => {
                        error!("Wallpaper could not be changed: Failed to start hyprpaper using Systemd and command line interface.");
                        return;
                    }
                },
            }
        }
        thread::sleep(Duration::from_millis(200));
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("unload")
            .arg("all")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        thread::sleep(Duration::from_millis(200));
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("preload")
            .arg(image.as_os_str())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        thread::sleep(Duration::from_millis(200));
        let monitor = if monitor == gettext("All") {
            ""
        } else {
            monitor
        };
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("wallpaper")
            .arg(format!("{monitor},{},{fit_mode}", image.to_str().unwrap()))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_hyprpaper_changer_bar(changer_specific_options_box: &Box, settings: &Settings) {
    let dropdown = DropDown::from_strings(&[
	&gettext("contain"),
	&gettext("cover"),
	&gettext("tile"),
	&gettext("fill")
    ]);
    dropdown.set_halign(Align::Start);
    dropdown.set_valign(Align::Center);
    dropdown.set_margin_top(DEFAULT_MARGIN);
    dropdown.set_margin_start(DEFAULT_MARGIN);
    dropdown.set_margin_bottom(DEFAULT_MARGIN);
    dropdown.set_margin_end(DEFAULT_MARGIN);
    changer_specific_options_box.append(&dropdown);
    settings.bind("hyprpaper-fit-mode", &dropdown, "selected").build();
}
