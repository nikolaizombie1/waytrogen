use std::{ffi::OsStr, fmt::Display, path::Path, process::Command, str::FromStr};
use strum_macros::EnumIter;

pub trait WallpaperChanger {
    fn change(&self, image: &Path, monitor: &str) -> anyhow::Result<()>;
    fn accepted_formats(&self) -> Vec<String>;
}

#[derive(EnumIter)]
pub enum WallpaperChangers {
    Hyprpaper,
    Swaybg
}

impl WallpaperChanger for WallpaperChangers {
    fn change(&self, image: &Path, monitor: &str) -> anyhow::Result<()> {
        match self {
            WallpaperChangers::Hyprpaper => {
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
            WallpaperChangers::Swaybg => Ok(()),
        }
    }

    fn accepted_formats(&self) -> Vec<String> {
        match self {
            WallpaperChangers::Hyprpaper => {
                vec![
                    "png".to_owned(),
                    "jpg".to_owned(),
                    "jpeg".to_owned(),
                    "webp".to_owned(),
                    "jxl".to_owned(),
                ]
            }
            WallpaperChangers::Swaybg => vec![
                    "png".to_owned(),
                    "jpg".to_owned(),
                    "jpeg".to_owned(),
                    "tiff".to_owned(),
                    "tga".to_owned(),
                    "gif".to_owned(),
            ],
        }
    }
}

impl FromStr for WallpaperChangers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "hyprpaper" => Ok(WallpaperChangers::Hyprpaper),
            "swaybg" => Ok(WallpaperChangers::Swaybg),
            _ => Err(format!("{} is not a valid wallpaper setter.", s)),
        }
    }
}

impl Display for WallpaperChangers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WallpaperChangers::Hyprpaper => write!(f, "Hyprpaper"),
            WallpaperChangers::Swaybg => write!(f, "swaybg"),
        }
    }
}
