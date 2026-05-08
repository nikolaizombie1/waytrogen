use crate::locale::TRANSLATION;
use iced::{
    Element,
    widget::{pick_list},
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
        let monitor = if monitor == TRANSLATION.get_translation("All") {
            ""
        } else {
            monitor
        };
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("wallpaper")
            .arg(format!(
                "{monitor},{},{}",
                image.to_str().unwrap(),
                settings.fit_mode
            ))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_hyprpaper_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let dropdown: Element<'_, Messages> = pick_list(
        HyprpaperFitModes::VARIANTS,
        app_state.hyprpaper_fill_mode.clone(),
        Messages::HyprpaperFitModeChanged,
    ).into();
    vec![dropdown]
}
