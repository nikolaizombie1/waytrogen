use lazy_static::lazy_static;
use regex::Regex;
use std::{ffi::OsStr, fmt::Display, path::Path, process::Command, str::FromStr};
use strum_macros::EnumIter;

pub trait WallpaperChanger {
    fn change(&self, image: &Path, monitor: &str) -> anyhow::Result<()>;
    fn accepted_formats(&self) -> Vec<String>;
}

#[derive(EnumIter, Clone)]
pub enum WallpaperChangers {
    Hyprpaper,
    Swaybg(SwaybgModes, String),
}

impl Default for WallpaperChangers {
    fn default() -> Self {
        WallpaperChangers::Hyprpaper
    }
}

#[derive(Clone)]
pub enum SwaybgModes {
    Stretch,
    Fit,
    Fill,
    Center,
    Tile,
    SolidColor,
}

impl Default for SwaybgModes {
    fn default() -> Self {
        SwaybgModes::Fill
    }
}

impl FromStr for SwaybgModes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase()[..] {
            "stretch" => Ok(SwaybgModes::Stretch),
            "fit" => Ok(SwaybgModes::Fit),
            "fill" => Ok(SwaybgModes::Fill),
            "center" => Ok(SwaybgModes::Center),
            "tile" => Ok(SwaybgModes::Tile),
            "solid_color" => Ok(SwaybgModes::SolidColor),
            _ => Err(format!("Unknown swaybg mode: {}", s)),
        }
    }
}

impl Display for SwaybgModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwaybgModes::Stretch => write!(f, "stretch"),
            SwaybgModes::Fit => write!(f, "fit"),
            SwaybgModes::Fill => write!(f, "fill"),
            SwaybgModes::Center => write!(f, "center"),
            SwaybgModes::Tile => write!(f, "tile"),
            SwaybgModes::SolidColor => write!(f, "solid_color"),
        }
    }
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
            WallpaperChangers::Swaybg(mode, rgb) => Ok(()),
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
            WallpaperChangers::Swaybg(_, _) => vec![
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

lazy_static! {
    static ref swaybg_regex: Regex =
        Regex::new(r"swaybg (stretch|fit|fill||center|tile|solid_color) [0-9a-f]{6}").unwrap();
}

impl FromStr for WallpaperChangers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "hyprland" => Ok(WallpaperChangers::Hyprpaper),
            _ if swaybg_regex.is_match(s) => {
                let args = s
                    .to_owned()
                    .split(" ")
                    .map(|s| s.to_owned())
                    .collect::<Vec<_>>();
                let mode = args[1].parse::<SwaybgModes>().unwrap();
                let rgb = args[2].clone();
                Ok(WallpaperChangers::Swaybg(mode, rgb))
            }
            _ => Err(format!("Unkown wallpaper changer: {}", s)),
        }
    }
}

impl Display for WallpaperChangers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WallpaperChangers::Hyprpaper => write!(f, "Hyprpaper"),
            WallpaperChangers::Swaybg(_, _) => write!(f, "swaybg"),
        }
    }
}
