use crate::{
    common::RGB, hyprpaper::change_hyprpaper_wallpaper, mpvpaper::change_mpvpaper_wallpaper,
    swaybg::change_swaybg_wallpaper, swww::change_swww_wallpaper,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, process::Command, str::FromStr, thread};
use strum::{IntoEnumIterator, VariantArray};
use strum_macros::{EnumIter, IntoStaticStr, VariantArray};
use which::which;

pub trait WallpaperChanger {
    fn change(self, image: PathBuf, monitor: String);
    fn accepted_formats(&self) -> Vec<String>;
    fn kill(&self);
}
pub trait U32Enum {
    fn from_u32(i: u32) -> Self;
    fn to_u32(&self) -> u32;
}

#[derive(Debug, EnumIter, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum WallpaperChangers {
    #[default]
    Hyprpaper,
    Swaybg(SwaybgModes, String),
    MpvPaper(MpvPaperPauseModes, MpvPaperSlideshowSettings, String),
    Swww(
        SWWWResizeMode,
        RGB,
        SWWWScallingFilter,
        SWWWTransitionType,
        u8,
        u32,
        u32,
        u16,
        SWWWTransitionPosition,
        bool,
        SWWWTransitionBezier,
        SWWWTransitionWave,
    ),
}

impl WallpaperChangers {
    pub fn killall_changers() {
        for changer in WallpaperChangers::iter() {
            changer.kill();
        }
    }
    fn kill_all_changers_except(changer: &WallpaperChangers) {
        let varient = match changer {
            Self::Hyprpaper => Self::Hyprpaper,
            Self::Swaybg(_, _) => Self::Swaybg(SwaybgModes::default(), String::default()),
            Self::MpvPaper(_, _, _) => Self::MpvPaper(
                MpvPaperPauseModes::default(),
                MpvPaperSlideshowSettings::default(),
                String::default(),
            ),
            Self::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => Self::Swww(
                SWWWResizeMode::default(),
                RGB::default(),
                SWWWScallingFilter::default(),
                SWWWTransitionType::default(),
                u8::default(),
                u32::default(),
                u32::default(),
                u16::default(),
                SWWWTransitionPosition::default(),
                bool::default(),
                SWWWTransitionBezier::default(),
                SWWWTransitionWave::default(),
            ),
        };
        WallpaperChangers::iter().for_each(|w| {
            if w != varient {
                w.kill();
            }
        });
    }
    #[must_use]
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

#[derive(Debug, Clone, IntoStaticStr, VariantArray, Default, Serialize, Deserialize, PartialEq)]
pub enum SwaybgModes {
    Stretch,
    Fit,
    #[default]
    Fill,
    Center,
    Tile,
    SolidColor,
}

#[derive(Debug, Clone, IntoStaticStr, VariantArray, Default, Serialize, Deserialize, PartialEq)]
pub enum MpvPaperPauseModes {
    None,
    #[default]
    AutoPause,
    AutoStop,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MpvPaperSlideshowSettings {
    pub enable: bool,
    pub seconds: u32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, VariantArray, PartialEq)]
pub enum SWWWResizeMode {
    No,
    #[default]
    Crop,
    Fit,
}

impl U32Enum for SWWWResizeMode {
    fn from_u32(i: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let i = i % Self::VARIANTS.len() as u32;
        match i {
            0 => Self::No,
            1 => Self::Crop,
            2 => Self::Fit,
            _ => Self::default(),
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Self::No => 0,
            Self::Crop => 1,
            Self::Fit => 2,
        }
    }
}

impl Display for SWWWResizeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::No => write!(f, "no"),
            Self::Crop => write!(f, "crop"),
            Self::Fit => write!(f, "fit"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, VariantArray, PartialEq)]
pub enum SWWWScallingFilter {
    Nearest,
    Bilinear,
    CatmullRom,
    Mitchell,
    #[default]
    Lanczos3,
}

impl U32Enum for SWWWScallingFilter {
    fn from_u32(i: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let i = i % Self::VARIANTS.len() as u32;
        match i {
            0 => Self::Nearest,
            1 => Self::Bilinear,
            2 => Self::CatmullRom,
            3 => Self::Mitchell,
            4 => Self::Lanczos3,
            _ => Self::default(),
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Self::Nearest => 0,
            Self::Bilinear => 1,
            Self::CatmullRom => 2,
            Self::Mitchell => 3,
            Self::Lanczos3 => 4,
        }
    }
}

impl Display for SWWWScallingFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nearest => write!(f, "Nearest"),
            Self::Bilinear => write!(f, "Bilinear"),
            Self::CatmullRom => write!(f, "CatmullRom"),
            Self::Mitchell => write!(f, "Mitchell"),
            Self::Lanczos3 => write!(f, "Lanczos3"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, VariantArray, PartialEq)]
pub enum SWWWTransitionType {
    None,
    #[default]
    Simple,
    Fade,
    Left,
    Right,
    Top,
    Bottom,
    Wipe,
    Wave,
    Grow,
    Center,
    Any,
    Outer,
    Random,
}

impl U32Enum for SWWWTransitionType {
    fn from_u32(i: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let i = i % Self::VARIANTS.len() as u32;
        match i {
            0 => Self::None,
            1 => Self::Simple,
            2 => Self::Fade,
            3 => Self::Left,
            4 => Self::Right,
            5 => Self::Top,
            6 => Self::Bottom,
            7 => Self::Wipe,
            8 => Self::Wave,
            9 => Self::Grow,
            10 => Self::Center,
            11 => Self::Any,
            12 => Self::Outer,
            13 => Self::Random,
            _ => Self::default(),
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Simple => 1,
            Self::Fade => 2,
            Self::Left => 3,
            Self::Right => 4,
            Self::Top => 5,
            Self::Bottom => 6,
            Self::Wipe => 7,
            Self::Wave => 8,
            Self::Grow => 9,
            Self::Center => 10,
            Self::Any => 11,
            Self::Outer => 12,
            Self::Random => 13,
        }
    }
}

impl Display for SWWWTransitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Simple => write!(f, "simple"),
            Self::Fade => write!(f, "fade"),
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
            Self::Top => write!(f, "top"),
            Self::Bottom => write!(f, "bottm"),
            Self::Wipe => write!(f, "wipe"),
            Self::Wave => write!(f, "wave"),
            Self::Grow => write!(f, "grow"),
            Self::Center => write!(f, "center"),
            Self::Any => write!(f, "any"),
            Self::Outer => write!(f, "outer"),
            Self::Random => write!(f, "random"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct SWWWTransitionPosition {
    pub position: String,
}

impl Display for SWWWTransitionPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

lazy_static! {
    static ref swww_transition_pos_regex: Regex =
        Regex::new(r"(0.\d\d?,0\.\d\d?)|(\d+,\d+)|(center|top|left|right|bottom|top-left|top-right|bottom-left|bottom-right)").unwrap();
}

impl SWWWTransitionPosition {
    pub fn new(s: &str) -> anyhow::Result<SWWWTransitionPosition> {
        if swww_transition_pos_regex.is_match(s) {
            Ok(Self {
                position: s.to_owned(),
            })
        } else {
            Err(anyhow::anyhow!("Invalid Transition Position"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SWWWTransitionBezier {
    pub p0: f64,
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
}

impl Display for SWWWTransitionBezier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.p0, self.p1, self.p2, self.p3)
    }
}

impl Default for SWWWTransitionBezier {
    fn default() -> Self {
        Self {
            p0: 0.54,
            p1: 0.0,
            p2: 0.34,
            p3: 0.99,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SWWWTransitionWave {
    pub width: u32,
    pub height: u32,
}

impl Display for SWWWTransitionWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.width, self.height)
    }
}

impl Default for SWWWTransitionWave {
    fn default() -> Self {
        Self {
            width: 20,
            height: 20,
        }
    }
}

impl U32Enum for SwaybgModes {
    fn from_u32(i: u32) -> SwaybgModes {
        let i = (i as usize) % SwaybgModes::VARIANTS.len();
        match i {
            1 => Self::Fit,
            2 => Self::Fill,
            3 => Self::Center,
            4 => Self::Tile,
            5 => Self::SolidColor,
            _ => Self::Stretch,
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Self::Stretch => 0,
            Self::Fit => 1,
            Self::Fill => 2,
            Self::Center => 3,
            Self::Tile => 4,
            Self::SolidColor => 5,
        }
    }
}

impl U32Enum for MpvPaperPauseModes {
    fn from_u32(i: u32) -> MpvPaperPauseModes {
        let i = (i as usize) % MpvPaperPauseModes::VARIANTS.len();
        match i {
            1 => Self::AutoPause,
            2 => Self::AutoStop,
            _ => Self::None,
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::AutoPause => 1,
            Self::AutoStop => 2,
        }
    }
}

impl FromStr for SwaybgModes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase()[..] {
            "stretch" => Ok(Self::Stretch),
            "fit" => Ok(Self::Fit),
            "fill" => Ok(Self::Fill),
            "center" => Ok(Self::Center),
            "tile" => Ok(Self::Tile),
            "solid_color" => Ok(Self::SolidColor),
            _ => Err(format!("Unknown swaybg mode: {s}")),
        }
    }
}

impl Display for SwaybgModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stretch => write!(f, "stretch"),
            Self::Fit => write!(f, "fit"),
            Self::Fill => write!(f, "fill"),
            Self::Center => write!(f, "center"),
            Self::Tile => write!(f, "tile"),
            Self::SolidColor => write!(f, "solid_color"),
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
        Self::kill_all_changers_except(&self);
        thread::spawn(move || match self {
            Self::Hyprpaper => {
                change_hyprpaper_wallpaper(&image, &monitor);
            }
            Self::Swaybg(_, _) => {
                change_swaybg_wallpaper(self, &image, &monitor);
            }
            Self::MpvPaper(_, _, _) => {
                change_mpvpaper_wallpaper(&self, image, &monitor);
            }
            Self::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {
                change_swww_wallpaper(self, image, monitor);
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
                let mut mpvpaper_formats = vec![
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
                ];
                let mut hyprpaper_formats = Self::Hyprpaper.accepted_formats();
                let mut swaybg_formats =
                    Self::Swaybg(SwaybgModes::Fill, "FFFFFF".to_owned()).accepted_formats();
                mpvpaper_formats.append(&mut hyprpaper_formats);
                mpvpaper_formats.append(&mut swaybg_formats);
                mpvpaper_formats
            }
            Self::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {
                vec![
                    "gif".to_owned(),
                    "jpeg".to_owned(),
                    "jpg".to_owned(),
                    "png".to_owned(),
                    "pnm".to_owned(),
                    "tga".to_owned(),
                    "tiff".to_owned(),
                    "webp".to_owned(),
                    "bmp".to_owned(),
                    "farbfeld".to_owned(),
                ]
            }
        }
    }
    fn kill(&self) {
        match self {
            Self::Hyprpaper => Command::new("pkill")
                .arg("-9")
                .arg("hyprpaper")
                .spawn()
                .unwrap()
                .wait()
                .unwrap(),
            Self::Swaybg(_, _) => Command::new("pkill")
                .arg("-9")
                .arg("swaybg")
                .spawn()
                .unwrap()
                .wait()
                .unwrap(),
            Self::MpvPaper(_, _, _) => Command::new("pkill")
                .arg("mpvpaper")
                .spawn()
                .unwrap()
                .wait()
                .unwrap(),
            Self::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => Command::new("pkill")
                .arg("-9")
                .arg("swww-daemon")
                .spawn()
                .unwrap()
                .wait()
                .unwrap(),
        };
    }
}

lazy_static! {
    static ref swaybg_regex: Regex =
        Regex::new(r"swaybg (stretch|fit|fill||center|tile|solid_color) [0-9a-f]{6}").unwrap();
}

impl Display for WallpaperChangers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hyprpaper => write!(f, "hyprpaper"),
            Self::Swaybg(_, _) => write!(f, "swaybg"),
            Self::MpvPaper(_, _, _) => write!(f, "mpvpaper"),
            Self::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => write!(f, "swww"),
        }
    }
}

pub fn get_available_wallpaper_changers() -> Vec<WallpaperChangers> {
    let mut available_changers = vec![];
    for changer in WallpaperChangers::iter() {
        match changer {
            WallpaperChangers::Hyprpaper => match which(
                WallpaperChangers::Hyprpaper
                    .to_string()
                    .to_ascii_uppercase(),
            ) {
                Ok(_) => available_changers.push(changer),
                Err(_) => {
                    if Command::new("systemctl")
                        .arg("--user")
                        .arg("list-unit-files")
                        .arg("hyprpaper.service")
                        .spawn()
                        .unwrap()
                        .wait()
                        .unwrap()
                        .success()
                    {
                        available_changers.push(changer);
                    }
                }
            },
            WallpaperChangers::Swaybg(_, _) => {
                append_changer_if_in_path(&mut available_changers, changer)
            }
            WallpaperChangers::MpvPaper(_, _, _) => {
                append_changer_if_in_path(&mut available_changers, changer)
            }
            WallpaperChangers::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {
                append_changer_if_in_path(&mut available_changers, changer)
            }
        }
    }
    available_changers
}

fn append_changer_if_in_path(
    available_changers: &mut Vec<WallpaperChangers>,
    changer: WallpaperChangers,
) {
    if which(changer.to_string().to_lowercase()).is_ok() {
        available_changers.push(changer);
    }
}
