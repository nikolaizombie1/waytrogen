use log::debug;
use std::{path::PathBuf, process::Command, thread, time::Duration};
use gettextrs::gettext;

pub fn change_hyprpaper_wallpaper(image: PathBuf, monitor: String) {
    debug!("Starting hyprpaper");
    if !Command::new("pgrep")
        .arg("hyprpaper")
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        #[allow(clippy::zombie_processes)]
        Command::new("hyprpaper").spawn().unwrap();
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
    let monitor = if monitor == gettext("All") { "" } else { &monitor };
    Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("wallpaper")
        .arg(format!("{},{}", monitor, image.to_str().unwrap()))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
