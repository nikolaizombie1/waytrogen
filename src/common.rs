use gtk::{glib::SignalHandlerId, Picture};
use image::ImageReader;
use lazy_static::lazy_static;
use log::trace;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::Display,
    io::Cursor,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::UNIX_EPOCH,
};

use crate::wallpaper_changers::WallpaperChangers;
use gettextrs::gettext;

pub const THUMBNAIL_HEIGHT: i32 = 200;
pub const THUMBNAIL_WIDTH: i32 = THUMBNAIL_HEIGHT;
pub const APP_ID: &str = "org.Waytrogen.Waytrogen";
pub const GETTEXT_DOMAIN: &str = "waytrogen";
pub const CONFIG_APP_NAME: &str = "waytrogen";
pub const CACHE_FILE_NAME: &str = "cache.db";
pub const CONFIG_FILE_NAME: &str = "config.json";

pub struct GtkPictureFile {
    pub picture: Picture,
    pub cache_image_file: CacheImageFile,
    pub button_signal_handler: RefCell<Option<SignalHandlerId>>,
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
        let temp_dir = String::from_utf8(Command::new("mktemp").arg("-d").output()?.stdout)?;
        let output_path = PathBuf::from(temp_dir.trim()).join("temp.png");
        trace!("ffmpeg Output Path: {}", output_path.to_str().unwrap());
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
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
            let red = f32::from(red[0]) / 255.0;
            let green = hex::decode(s[2..=3].iter().collect::<String>()).unwrap();
            let green = f32::from(green[0]) / 255.0;
            let blue = hex::decode(s[4..=5].iter().collect::<String>()).unwrap();
            let blue = f32::from(blue[0]) / 255.0;
            Ok(Self { red, green, blue })
        } else {
            Err(gettext("Invalid string"))
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Wallpaper {
    pub monitor: String,
    pub path: String,
    pub changer: WallpaperChangers,
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn sort_by_sort_dropdown_string(files: &mut [PathBuf], sort_by: &str, invert_sort: bool) {
    match sort_by {
        "name" => {
            files.sort_by(|f1, f2| {
                if invert_sort {
                    f1.file_name().partial_cmp(&f2.file_name()).unwrap()
                } else {
                    f2.file_name().partial_cmp(&f1.file_name()).unwrap()
                }
            });
        }
        "date" => {
            files.sort_by(|f1, f2| {
                if invert_sort {
                    f1.metadata()
                        .unwrap()
                        .created()
                        .unwrap()
                        .partial_cmp(&f2.metadata().unwrap().created().unwrap())
                        .unwrap()
                } else {
                    f2.metadata()
                        .unwrap()
                        .created()
                        .unwrap()
                        .partial_cmp(&f1.metadata().unwrap().created().unwrap())
                        .unwrap()
                }
            });
        }
        _ => {}
    }
}

pub fn parse_executable_script(s: &str) -> anyhow::Result<String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    let path = s.parse::<PathBuf>()?;
    if !path.metadata()?.is_file() {
        return Err(anyhow::anyhow!("Input is not a file"));
    }
    if path.metadata()?.permissions().mode() & 0o111 == 0 {
        return Err(anyhow::anyhow!("File is not executable"));
    }
    Ok(s.to_owned())
}
