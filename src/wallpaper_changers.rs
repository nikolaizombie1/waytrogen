use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr, fmt::Display, path::PathBuf, process::Command, str::FromStr, thread, time::Duration,
};
use strum::{IntoEnumIterator, VariantArray};
use strum_macros::{EnumIter, IntoStaticStr, VariantArray};

pub trait WallpaperChanger {
    fn change(self, image: PathBuf, monitor: String);
    fn accepted_formats(&self) -> Vec<String>;
}

#[derive(Debug, EnumIter, Clone, Default, Serialize, Deserialize)]
pub enum WallpaperChangers {
    #[default]
    Hyprpaper,
    Swaybg(SwaybgModes, String),
    MpvPaper(MpvPaperPauseModes, MpvPaperSlideshowSettings, String),
}

impl WallpaperChangers {
    pub fn killall_changers() {
        thread::spawn(|| {
            for changer in WallpaperChangers::iter() {
                Command::new("pkill")
                    .arg(changer.to_string())
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        });
    }
    pub fn all_accepted_formats() -> Vec<String> {
        let mut accepted_formats = vec![];
        for changer in WallpaperChangers::iter() {
            for format in changer.accepted_formats() {
                if !accepted_formats.iter().any(|f: &String| *f == format) {
                    accepted_formats.push(format);
                }
            }
        }
        accepted_formats
    }
}

#[derive(Debug, Clone, IntoStaticStr, VariantArray, Default, Serialize, Deserialize)]
pub enum SwaybgModes {
    Stretch,
    Fit,
    #[default]
    Fill,
    Center,
    Tile,
    SolidColor,
}

#[derive(Debug, Clone, IntoStaticStr, VariantArray, Default, Serialize, Deserialize)]
pub enum MpvPaperPauseModes {
    None,
    #[default]
    AutoPause,
    AutoStop,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MpvPaperSlideshowSettings {
    pub enable: bool,
    pub seconds: u32,
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

impl MpvPaperPauseModes {
    pub fn from_u32(i: u32) -> MpvPaperPauseModes {
        let i = (i as usize) % MpvPaperPauseModes::VARIANTS.len();
        match i {
            0 => MpvPaperPauseModes::None,
            1 => MpvPaperPauseModes::AutoPause,
            2 => MpvPaperPauseModes::AutoStop,
            _ => MpvPaperPauseModes::None,
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

impl FromStr for MpvPaperPauseModes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "none" => Ok(Self::None),
            "auto-pause" => Ok(Self::AutoPause),
            "auto-stop" => Ok(Self::AutoStop),
            _ => Err("Invalid pause mode".to_owned()),
        }
    }
}

impl Display for MpvPaperPauseModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpvPaperPauseModes::None => write!(f, "none"),
            MpvPaperPauseModes::AutoPause => write!(f, "auto-pause"),
            MpvPaperPauseModes::AutoStop => write!(f, "auto-stop"),
        }
    }
}

impl WallpaperChanger for WallpaperChangers {
    fn change(self, image: PathBuf, monitor: String) {
        thread::spawn(move || match self {
            Self::Hyprpaper => {
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
            Self::Swaybg(mode, rgb) => {
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
                    .unwrap()
                    .wait()
                    .unwrap();
            }
            Self::MpvPaper(pause_mode, slideshow, mpv_options) => {
                let mut command = Command::new("mpvpaper");
                command.arg("-o").arg(format!("\"{}\"",mpv_options));
                match pause_mode {
                    MpvPaperPauseModes::None => {}
                    MpvPaperPauseModes::AutoPause => {
                        command.arg("--auto-pause");
                    }
                    MpvPaperPauseModes::AutoStop => {
                        command.arg("--auto-stop");
                    }
                }
                if slideshow.enable {
                    command.arg("-n").arg(slideshow.seconds.to_string());
                }
                command
                    .arg("-f")
                    .arg(monitor)
                    .arg(image)
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        });
    }

    fn accepted_formats(&self) -> Vec<String> {
        match self {
            Self::Hyprpaper => {
                vec![
                    "png".to_owned(),
                    "jpg".to_owned(),
                    "jpeg".to_owned(),
                    "webp".to_owned(),
                    "jxl".to_owned(),
                ]
            }
            Self::Swaybg(_, _) => vec![
                "png".to_owned(),
                "jpg".to_owned(),
                "jpeg".to_owned(),
                "tiff".to_owned(),
                "tif".to_owned(),
                "tga".to_owned(),
                "gif".to_owned(),
            ],
            Self::MpvPaper(_, _, _) => {
                vec![
                    "str".to_owned(),
                    "aa".to_owned(),
                    "aac".to_owned(),
                    "aax".to_owned(),
                    "ac3".to_owned(),
                    "ac4".to_owned(),
                    "acm".to_owned(),
                    "adf".to_owned(),
                    "adp".to_owned(),
                    "dtk".to_owned(),
                    "ads".to_owned(),
                    "ss2".to_owned(),
                    "adx".to_owned(),
                    "aea".to_owned(),
                    "afc".to_owned(),
                    "aix".to_owned(),
                    "al".to_owned(),
                    "apc".to_owned(),
                    "ape".to_owned(),
                    "apl".to_owned(),
                    "mac".to_owned(),
                    "aptx".to_owned(),
                    "aptxhd".to_owned(),
                    "aqt".to_owned(),
                    "ast".to_owned(),
                    "obu".to_owned(),
                    "avi".to_owned(),
                    "avs".to_owned(),
                    "avr".to_owned(),
                    "avs".to_owned(),
                    "avs2".to_owned(),
                    "avs3".to_owned(),
                    "bfstm".to_owned(),
                    "bcstm".to_owned(),
                    "binka".to_owned(),
                    "bit".to_owned(),
                    "bitpacked".to_owned(),
                    "bmv".to_owned(),
                    "bonk".to_owned(),
                    "brstm".to_owned(),
                    "avs".to_owned(),
                    "cdg".to_owned(),
                    "cdxl".to_owned(),
                    "xl".to_owned(),
                    "c2".to_owned(),
                    "302".to_owned(),
                    "daud".to_owned(),
                    "str".to_owned(),
                    "adp".to_owned(),
                    "dfpwm".to_owned(),
                    "dav".to_owned(),
                    "dss".to_owned(),
                    "dts".to_owned(),
                    "dtshd".to_owned(),
                    "dv".to_owned(),
                    "dif".to_owned(),
                    "cdata".to_owned(),
                    "eac3".to_owned(),
                    "ec3".to_owned(),
                    "paf".to_owned(),
                    "fap".to_owned(),
                    "evc".to_owned(),
                    "flm".to_owned(),
                    "flac".to_owned(),
                    "flv".to_owned(),
                    "fsb".to_owned(),
                    "fwse".to_owned(),
                    "g722".to_owned(),
                    "722".to_owned(),
                    "tco".to_owned(),
                    "rco".to_owned(),
                    "g723_1".to_owned(),
                    "g729".to_owned(),
                    "genh".to_owned(),
                    "gif".to_owned(),
                    "gsm".to_owned(),
                    "h261".to_owned(),
                    "h26l".to_owned(),
                    "h264".to_owned(),
                    "264".to_owned(),
                    "avc".to_owned(),
                    "hca".to_owned(),
                    "hevc".to_owned(),
                    "h265".to_owned(),
                    "265".to_owned(),
                    "iamf".to_owned(),
                    "idf".to_owned(),
                    "ifv".to_owned(),
                    "cgi".to_owned(),
                    "ipu".to_owned(),
                    "sf".to_owned(),
                    "ircam".to_owned(),
                    "ivr".to_owned(),
                    "jxl".to_owned(),
                    "kux".to_owned(),
                    "laf".to_owned(),
                    "lc3".to_owned(),
                    "669".to_owned(),
                    "abc".to_owned(),
                    "amf".to_owned(),
                    "ams".to_owned(),
                    "dbm".to_owned(),
                    "dmf".to_owned(),
                    "dsm".to_owned(),
                    "far".to_owned(),
                    "it".to_owned(),
                    "mdl".to_owned(),
                    "med".to_owned(),
                    "mid".to_owned(),
                    "mod".to_owned(),
                    "mt2".to_owned(),
                    "mtm".to_owned(),
                    "okt".to_owned(),
                    "psm".to_owned(),
                    "ptm".to_owned(),
                    "s3m".to_owned(),
                    "stm".to_owned(),
                    "ult".to_owned(),
                    "umx".to_owned(),
                    "xm".to_owned(),
                    "itgz".to_owned(),
                    "itr".to_owned(),
                    "itz".to_owned(),
                    "mdgz".to_owned(),
                    "mdr".to_owned(),
                    "mdz".to_owned(),
                    "s3gz".to_owned(),
                    "s3r".to_owned(),
                    "s3z".to_owned(),
                    "xmgz".to_owned(),
                    "xmr".to_owned(),
                    "xmz".to_owned(),
                    "669".to_owned(),
                    "amf".to_owned(),
                    "ams".to_owned(),
                    "dbm".to_owned(),
                    "digi".to_owned(),
                    "dmf".to_owned(),
                    "dsm".to_owned(),
                    "dtm".to_owned(),
                    "far".to_owned(),
                    "gdm".to_owned(),
                    "ice".to_owned(),
                    "imf".to_owned(),
                    "it".to_owned(),
                    "j2b".to_owned(),
                    "m15".to_owned(),
                    "mdl".to_owned(),
                    "med".to_owned(),
                    "mmcmp".to_owned(),
                    "mms".to_owned(),
                    "mo3".to_owned(),
                    "mod".to_owned(),
                    "mptm".to_owned(),
                    "mt2".to_owned(),
                    "mtm".to_owned(),
                    "nst".to_owned(),
                    "okt".to_owned(),
                    "plm".to_owned(),
                    "ppm".to_owned(),
                    "psm".to_owned(),
                    "pt36".to_owned(),
                    "ptm".to_owned(),
                    "s3m".to_owned(),
                    "sfx".to_owned(),
                    "sfx2".to_owned(),
                    "st26".to_owned(),
                    "stk".to_owned(),
                    "stm".to_owned(),
                    "stp".to_owned(),
                    "ult".to_owned(),
                    "umx".to_owned(),
                    "wow".to_owned(),
                    "xm".to_owned(),
                    "xpk".to_owned(),
                    "flv".to_owned(),
                    "dat".to_owned(),
                    "lvf".to_owned(),
                    "m4v".to_owned(),
                    "mkv".to_owned(),
                    "mk3d".to_owned(),
                    "mka".to_owned(),
                    "mks".to_owned(),
                    "webm".to_owned(),
                    "mca".to_owned(),
                    "mcc".to_owned(),
                    "mjpg".to_owned(),
                    "mjpeg".to_owned(),
                    "mpo".to_owned(),
                    "j2k".to_owned(),
                    "mlp".to_owned(),
                    "mods".to_owned(),
                    "moflex".to_owned(),
                    "mov".to_owned(),
                    "mp4".to_owned(),
                    "m4a".to_owned(),
                    "3gp".to_owned(),
                    "3g2".to_owned(),
                    "mj2".to_owned(),
                    "psp".to_owned(),
                    "m4b".to_owned(),
                    "ism".to_owned(),
                    "ismv".to_owned(),
                    "isma".to_owned(),
                    "f4v".to_owned(),
                    "avif".to_owned(),
                    "heic".to_owned(),
                    "heif".to_owned(),
                    "mp2".to_owned(),
                    "mp3".to_owned(),
                    "m2a".to_owned(),
                    "mpa".to_owned(),
                    "mpc".to_owned(),
                    "mjpg".to_owned(),
                    "txt".to_owned(),
                    "mpl2".to_owned(),
                    "sub".to_owned(),
                    "msf".to_owned(),
                    "mtaf".to_owned(),
                    "ul".to_owned(),
                    "musx".to_owned(),
                    "mvi".to_owned(),
                    "mxg".to_owned(),
                    "v".to_owned(),
                    "nist".to_owned(),
                    "sph".to_owned(),
                    "nsp".to_owned(),
                    "nut".to_owned(),
                    "obu".to_owned(),
                    "ogg".to_owned(),
                    "oma".to_owned(),
                    "omg".to_owned(),
                    "aa3".to_owned(),
                    "osq".to_owned(),
                    "pdv".to_owned(),
                    "pjs".to_owned(),
                    "pvf".to_owned(),
                    "qoa".to_owned(),
                    "yuv".to_owned(),
                    "cif".to_owned(),
                    "qcif".to_owned(),
                    "rgb".to_owned(),
                    "rt".to_owned(),
                    "rsd".to_owned(),
                    "rka".to_owned(),
                    "rsd".to_owned(),
                    "rso".to_owned(),
                    "sw".to_owned(),
                    "sb".to_owned(),
                    "smi".to_owned(),
                    "sami".to_owned(),
                    "sbc".to_owned(),
                    "msbc".to_owned(),
                    "sbg".to_owned(),
                    "scc".to_owned(),
                    "sdns".to_owned(),
                    "sdr2".to_owned(),
                    "sds".to_owned(),
                    "sdx".to_owned(),
                    "ser".to_owned(),
                    "sga".to_owned(),
                    "shn".to_owned(),
                    "vb".to_owned(),
                    "son".to_owned(),
                    "imx".to_owned(),
                    "sln".to_owned(),
                    "mjpg".to_owned(),
                    "stl".to_owned(),
                    "sub".to_owned(),
                    "sub".to_owned(),
                    "sup".to_owned(),
                    "svag".to_owned(),
                    "svs".to_owned(),
                    "tak".to_owned(),
                    "thd".to_owned(),
                    "tta".to_owned(),
                    "ans".to_owned(),
                    "art".to_owned(),
                    "asc".to_owned(),
                    "diz".to_owned(),
                    "ice".to_owned(),
                    "nfo".to_owned(),
                    "txt".to_owned(),
                    "vt".to_owned(),
                    "ty".to_owned(),
                    "ty+".to_owned(),
                    "uw".to_owned(),
                    "ub".to_owned(),
                    "usm".to_owned(),
                    "v210".to_owned(),
                    "yuv10".to_owned(),
                    "vag".to_owned(),
                    "vc1".to_owned(),
                    "rcv".to_owned(),
                    "viv".to_owned(),
                    "idx".to_owned(),
                    "vpk".to_owned(),
                    "txt".to_owned(),
                    "vqf".to_owned(),
                    "vql".to_owned(),
                    "vqe".to_owned(),
                    "h266".to_owned(),
                    "266".to_owned(),
                    "vvc".to_owned(),
                    "way".to_owned(),
                    "wa".to_owned(),
                    "vtt".to_owned(),
                    "wsd".to_owned(),
                    "xmd".to_owned(),
                    "xmv".to_owned(),
                    "xvag".to_owned(),
                    "yop".to_owned(),
                    "y4m".to_owned(),
                ]
            }
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
                let args = s.split(" ").map(|s| s.to_owned()).collect::<Vec<_>>();
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
            Self::Hyprpaper => write!(f, "Hyprpaper"),
            Self::Swaybg(_, _) => write!(f, "swaybg"),
            Self::MpvPaper(_, _, _) => write!(f, "mpvpaper"),
        }
    }
}
