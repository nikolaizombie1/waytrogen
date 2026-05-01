use gettextrs::gettext;
use iced::{Element, widget::{PickList, pick_list}};
use log::{debug, error, warn};
use strum::VariantArray;
use std::{path::Path, process::Command, thread, time::Duration};
use which::which;

use crate::{app_state::{self, AppState, Messages}, common::DEFAULT_MARGIN, wallpaper_changers::{HyprpaperFitModes, WallpaperChangers}};

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
                        warn!(
                            "Hyprpaper could not be started using Systemd. Attempting to start using command line interface"
                        );
                        #[allow(clippy::zombie_processes)]
                        Command::new("hyprpaper").spawn().unwrap();
                    }
                    Err(_) => {
                        error!(
                            "Wallpaper could not be changed: Failed to start hyprpaper using Systemd and command line interface."
                        );
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

pub fn generate_hyprpaper_changer_bar(app_state:  &AppState) -> Element<'_, Messages> {
    let dropdown = pick_list(
	HyprpaperFitModes::VARIANTS,
	app_state.hyprpaper_fill_mode.clone(),
	Messages::HyprpaperFitModeChanged
    );
    dropdown.into()
}
