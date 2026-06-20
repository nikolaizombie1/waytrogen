use crate::{common::create_tooltip, locale::TRANSLATION, monitors::AvailableMonitors};
use iced::{
    Element,
    widget::{pick_list, text},
};
use log::{debug, error, warn};
use std::{path::Path, process::Command, thread, time::Duration};
use strum::VariantArray;
use which::which;

use crate::{
    app_state::{AppState, Messages},
    wallpaper_changers::{HyprpaperFitModes, WallpaperChangers},
};

pub fn change_hyprpaper_wallpaper(
    hyprpaper_changer: WallpaperChangers,
    image: &Path,
    monitor: &str,
) {
    if let WallpaperChangers::Hyprpaper(settings) = hyprpaper_changer {
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
                .wait_with_output()
            {
                Ok(_) => {}
                Err(_) => {
                    if which("hyprpaper").is_ok() {
                        warn!(
                            "Hyprpaper could not be started using Systemd. Attempting to start using command line interface"
                        );
                        thread::spawn(|| {
                            Command::new("hyprpaper")
                                .spawn()
                                .unwrap()
                                .wait_with_output()
                                .unwrap();
                        });
                    } else {
                        error!(
                            "Wallpaper could not be changed: Failed to start hyprpaper using Systemd and command line interface."
                        );
                        return;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
        let fit_mode = match settings.fit_mode {
            HyprpaperFitModes::Contain => "contain",
            HyprpaperFitModes::Cover => "cover",
            HyprpaperFitModes::Tile => "tile",
            HyprpaperFitModes::Fill => "fill",
        };
        if monitor == TRANSLATION.get_translation("All") {
            if let Ok(available_monitors) = AvailableMonitors::get_monitors() {
                for monitor in available_monitors
                    .available_monitors
                    .into_iter()
                    .filter(|m| m != &TRANSLATION.get_translation("All"))
                {
                    Command::new("hyprctl")
                        .arg("hyprpaper")
                        .arg("wallpaper")
                        .arg(format!(
                            "{monitor},{},{}",
                            image.to_str().unwrap(),
                            fit_mode
                        ))
                        .spawn()
                        .unwrap()
                        .wait_with_output()
                        .unwrap();
                    thread::sleep(Duration::from_millis(200));
                }
            }
        } else {
            Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(format!(
                    "{monitor},{},{}",
                    image.to_str().unwrap(),
                    fit_mode
                ))
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
        }
    }
}

pub fn generate_hyprpaper_changer_bar(app_state: &AppState) -> Vec<Element<'static, Messages>> {
    let dropdown: Element<'_, Messages> = create_tooltip(
        pick_list(
            HyprpaperFitModes::VARIANTS,
            app_state.hyprpaper_fill_mode.clone(),
            Messages::HyprpaperFitModeChanged,
        )
        .into(),
        text![
            "{}",
            TRANSLATION.get_translation("hyprpaper-fit-mode-tooltip")
        ]
        .into(),
    )
    .into();
    vec![dropdown]
}
