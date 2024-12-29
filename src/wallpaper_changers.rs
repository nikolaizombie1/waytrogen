use gtk::prelude::*;
use image::ImageFormat;
use std::{ffi::OsStr, path::Path, process::Command};
pub trait WallpaperChanger {
    fn change(image: &Path, monitor: &str) -> anyhow::Result<()>;
    fn accepted_formats() -> Vec<ImageFormat>;
}

pub struct Hyprpaper {}

impl WallpaperChanger for Hyprpaper {
    fn change(image: &Path, monitor: &str) -> anyhow::Result<()> {
        let mut system = sysinfo::System::new();
        system.refresh_all();
        if system
            .processes_by_name(OsStr::new("hyprpaper"))
            .collect::<Vec<_>>()
            .is_empty()
        {
            Command::new("hyprpaper").spawn()?;
        }
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("unload")
            .arg("all")
            .spawn()?
            .wait()?;
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("preload")
            .arg(image.as_os_str())
            .spawn()?
            .wait()?;
        Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("wallpaper")
            .arg(format!("{},{}", monitor, image.to_str().unwrap()))
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn accepted_formats() -> Vec<ImageFormat> {
        vec![ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::WebP]
    }
}
