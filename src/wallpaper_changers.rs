use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::{
    ffi::OsStr,
    fmt::Display,
    path::PathBuf,
    process::Command,
    str::FromStr,
    thread, time::Duration,
};
use strum::{IntoEnumIterator, VariantArray};
use strum_macros::{EnumIter, IntoStaticStr, VariantArray};

pub trait WallpaperChanger {
    fn change(self, image: PathBuf, monitor: String);
    fn accepted_formats(&self) -> Vec<String>;
}

#[derive(EnumIter, Clone)]
#[derive(Default)]
pub enum WallpaperChangers {
    #[default]
    Hyprpaper,
    Swaybg(SwaybgModes, String),
}

impl WallpaperChangers {
    pub fn killall_changers() {
        thread::spawn(|| {
            for changer in WallpaperChangers::iter() {
                Command::new("pkill")
                    .arg(changer.to_string())
                    .spawn()
                    .unwrap().wait().unwrap();
            }
        });
    }
}


#[derive(Clone, IntoStaticStr, VariantArray)]
#[derive(Default)]
pub enum SwaybgModes {
    Stretch,
    Fit,
    #[default]
    Fill,
    Center,
    Tile,
    SolidColor,
}

impl SwaybgModes {
    pub fn from_u32(i: u32) -> SwaybgModes {
        let i = (i as usize) % SwaybgModes::VARIANTS.len();
        match i {
            0 => SwaybgModes::Stretch,
            1 => SwaybgModes::Fit,
            2 => SwaybgModes::Fill,
            3 => SwaybgModes::Center,
            4 => SwaybgModes::Tile,
            5 => SwaybgModes::SolidColor,
            _ => SwaybgModes::Stretch,
        }
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
    fn change(self, image: PathBuf, monitor: String) {
        thread::spawn(move || match self {
            WallpaperChangers::Hyprpaper => {
                let mut system = sysinfo::System::new();
                system.refresh_all();
                if system
                    .processes_by_name(OsStr::new("hyprpaper"))
                    .collect::<Vec<_>>()
                    .is_empty()
                {
                    debug!("Starting hyprpaper");
                    Command::new("hyprpaper").spawn().unwrap().wait().unwrap();
                }
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
                Command::new("hyprctl")
                    .arg("hyprpaper")
                    .arg("wallpaper")
                    .arg(format!("{},{}", monitor, image.to_str().unwrap()))
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
            WallpaperChangers::Swaybg(mode, rgb) => {
                Command::new("swaybg")
                    .arg("-c")
                    .arg(rgb)
                    .arg("-i")
                    .arg(image.to_str().unwrap())
                    .arg("-m")
                    .arg(mode.to_string())
                    .arg("-o")
                    .arg(monitor)
                    .spawn()
                    .unwrap().wait().unwrap();
            }
        });
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
            "hyprpaper" => Ok(WallpaperChangers::Hyprpaper),
            _ if swaybg_regex.is_match(s) => {
                let args = s
                    .split(" ")
                    .map(|s| s.to_owned())
                    .collect::<Vec<_>>();
                let mode = args[1].parse::<SwaybgModes>().unwrap();
                let rgb = args[2].clone();
                Ok(WallpaperChangers::Swaybg(mode, rgb))
            }
            _ => Err(format!("Unknown wallpaper changer: {}", s)),
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
