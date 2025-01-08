use clap::Parser;
use gtk::Picture;
use image::ImageReader;
use lazy_static::lazy_static;
use mktemp::Temp;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::UNIX_EPOCH,
};

use crate::wallpaper_changers::WallpaperChangers;
use gettextrs::*;

pub const THUMBNAIL_HEIGHT: i32 = 200;
pub const THUMBNAIL_WIDTH: i32 = THUMBNAIL_HEIGHT;

#[derive(Clone)]
pub struct GtkPictureFile {
    pub picture: Picture,
    pub chache_image_file: CacheImageFile,
}

#[derive(Clone, Default, PartialEq)]
pub struct CacheImageFile {
    pub image: Vec<u8>,
    pub name: String,
    pub date: u64,
    pub path: String,
}

impl CacheImageFile {
    pub fn from_file(path: &Path) -> anyhow::Result<CacheImageFile> {
        let image = Self::generate_thumbnail(path)?;
        Self::create_gtk_image(path, image)
    }

    fn get_metadata(path: &Path) -> anyhow::Result<(String, String, u64)> {
        let path = path.to_path_buf();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let date = std::fs::File::open(path.clone())?.metadata()?.modified()?;
        let date = date.duration_since(UNIX_EPOCH)?.as_secs();
        Ok((path.to_str().unwrap().to_string(), name, date))
    }

    fn create_gtk_image(path: &Path, image: Vec<u8>) -> anyhow::Result<CacheImageFile> {
        let fields = Self::get_metadata(path)?;
        let image_file = CacheImageFile {
            image,
            path: fields.0,
            name: fields.1,
            date: fields.2,
        };
        Ok(image_file)
    }

    fn generate_thumbnail(path: &Path) -> anyhow::Result<Vec<u8>> {
        if let Ok(i) = Self::try_create_thumbnail_with_image(path) {
            return Ok(i);
        }
        if let Ok(i) = Self::try_create_thumbnail_with_ffmpeg(path) {
            return Ok(i);
        }
        Err(anyhow::anyhow!(
            "{}: {}",
            gettext("Failed to create thumbnail for"),
            path.as_os_str().to_str().unwrap_or_default()
        ))
    }
    fn try_create_thumbnail_with_ffmpeg(path: &Path) -> anyhow::Result<Vec<u8>> {
        let temp_dir = Temp::new_dir()?;
        let output_path = PathBuf::from(temp_dir.as_os_str()).join("temp.png");
        let code = Command::new("ffmpeg")
            .arg("-i")
            .arg(path)
            .arg("-y")
            .arg("-ss")
            .arg("00:00:00")
            .arg("-frames:v")
            .arg("1")
            .arg(output_path.clone())
            .spawn()?
            .wait()?
            .code()
            .unwrap_or(255);
        match code {
            0 => Self::try_create_thumbnail_with_image(&output_path),
            _ => Err(anyhow::anyhow!(gettext(
                "Thumbnail could not be generated using ffmpg."
            ))),
        }
    }

    fn try_create_thumbnail_with_image(path: &Path) -> anyhow::Result<Vec<u8>> {
        let thumbnail = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?
            .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
            .to_rgb8();
        let mut buff: Vec<u8> = vec![];
        thumbnail.write_to(&mut Cursor::new(&mut buff), image::ImageFormat::Png)?;
        Ok(buff)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
impl Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}",
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8
        )
    }
}

lazy_static! {
    static ref rgb_regex: Regex = Regex::new(r"[0-9A-Fa-f]{6}").unwrap();
}

impl FromStr for RGB {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if rgb_regex.is_match(s) {
            let s = s.to_lowercase().chars().collect::<Vec<_>>();
            let red = hex::decode(s[0..=1].iter().collect::<String>()).unwrap();
            let red = (red[0] as f32) / 255.0;
            let green = hex::decode(s[2..=3].iter().collect::<String>()).unwrap();
            let green = (green[0] as f32) / 255.0;
            let blue = hex::decode(s[4..=5].iter().collect::<String>()).unwrap();
            let blue = (blue[0] as f32) / 255.0;
            Ok(Self { red, green, blue })
        } else {
            Err(gettext("Invalid string"))
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Wallpaper {
    pub monitor: String,
    pub path: String,
    pub changer: WallpaperChangers,
}

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    /// Restore previously set wallpapers
    pub restore: bool,
    #[arg(short, long, default_value_t = 0)]
    /// How many error, warning, info, debug or trace logs will be shown. 0 for error, 1 for warning, 2 for info, 3 for debug, 4 or higher for trace.
    pub verbosity: u8,
}
