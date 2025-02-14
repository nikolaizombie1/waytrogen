use gettextrs::gettext;
use log::{debug, error, warn};
use std::{path::Path, process::Command, thread, time::Duration};
use which::which;

pub fn change_hyprpaper_wallpaper(image: &Path, monitor: &str) {
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
        .arg(format!("{},{}", monitor, image.to_str().unwrap()))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
