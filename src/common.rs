use crate::wallpaper_changers::WallpaperChangers;
use crate::{app_state::Messages, locale::TRANSLATION};
use anyhow::anyhow;
use iced::widget::{Tooltip, container, tooltip};
use iced::{Element, Renderer, Theme};
use image::ImageReader;
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Cursor,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::app_state::SortBy;

pub const DEFAULT_MARGIN: f32 = 12.0;
pub const THUMBNAIL_HEIGHT: i32 = 400;
pub const THUMBNAIL_WIDTH: i32 = THUMBNAIL_HEIGHT;
pub const BUTTON_HEIGHT: f32 = 200.0;
pub const BUTTON_WIDTH: f32 = BUTTON_HEIGHT;
pub const APP_ID: &str = "org.Waytrogen.Waytrogen";
pub const GETTEXT_DOMAIN: &str = "waytrogen";
pub const CONFIG_APP_NAME: &str = "waytrogen";
pub const CACHE_FILE_NAME: &str = "cache.db";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const DEFAULT_TOOLTIP_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Default, PartialEq, Hash)]
pub struct CacheImageFile {
    pub cached_image_path: PathBuf,
    pub name: String,
    pub date: u32,
    pub path: PathBuf,
    pub favorite: bool,
}

impl CacheImageFile {
    pub fn from_file(path: &Path) -> anyhow::Result<CacheImageFile> {
        let image = Self::generate_thumbnail(path)?;
        Self::create_gtk_image(path, &image)
    }

    fn get_metadata(path: &Path) -> anyhow::Result<(String, String, u32)> {
        let path = path.to_path_buf();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let date = std::fs::File::open(path.clone())?.metadata()?.modified()?;
        let date = u32::try_from(date.duration_since(UNIX_EPOCH)?.as_secs())?;
        Ok((path.to_str().unwrap().to_string(), name, date))
    }

    fn create_gtk_image(path: &Path, image: &Path) -> anyhow::Result<CacheImageFile> {
        let fields = Self::get_metadata(path)?;
        let image_file = CacheImageFile {
            cached_image_path: image.to_path_buf(),
            path: PathBuf::from(fields.0),
            name: fields.1,
            date: fields.2,
            favorite: false,
        };
        Ok(image_file)
    }

    fn generate_thumbnail(path: &Path) -> anyhow::Result<PathBuf> {
        if let Ok(i) = Self::try_write_thumbnail_with_image(path) {
            return Ok(i);
        }
        if let Ok(i) = Self::try_write_thumbnail_with_ffmpeg(path) {
            return Ok(i);
        }
        Err(anyhow::anyhow!(
            "{}: {}",
            TRANSLATION.get_translation("failed-to-create-thumbnail-for"),
            path.as_os_str().to_str().unwrap_or_default()
        ))
    }
    fn try_write_thumbnail_with_ffmpeg(path: &Path) -> anyhow::Result<PathBuf> {
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
            0 => Self::try_write_thumbnail_with_image(&output_path),
            _ => Err(anyhow::anyhow!(TRANSLATION.get_translation(
                "Thumbnail could not be generated using ffmpg."
            ))),
        }
    }

    fn try_write_thumbnail_with_image(path: &Path) -> anyhow::Result<PathBuf> {
        let thumbnail = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?
            .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
            .to_rgb8();
        let image_name = format!("{}.png", Uuid::new_v4());
        let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME);
        let Some(cache_dir) = xdg_dirs.get_cache_home() else {
            return Err(anyhow!("Failed to get cache directory"));
        };
        let image_file = cache_dir.join(Path::new(&image_name));
        let mut buff: Vec<u8> = vec![];
        thumbnail.write_to(&mut Cursor::new(&mut buff), image::ImageFormat::Png)?;
        fs::write(&image_file, buff)?;
        Ok(image_file)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Wallpaper {
    pub monitor: String,
    pub path: String,
    pub changer: WallpaperChangers,
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn sort_by_sort_dropdown_string(files: &mut [PathBuf], sort_by: &SortBy, invert_sort: bool) {
    match sort_by {
        SortBy::Name => {
            files.sort_by(|f1, f2| {
                if invert_sort {
                    f1.file_name().partial_cmp(&f2.file_name()).unwrap()
                } else {
                    f2.file_name().partial_cmp(&f1.file_name()).unwrap()
                }
            });
        }
        SortBy::Date => {
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

pub fn get_config_file_path() -> anyhow::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME);
    let config_file = xdg_dirs.place_config_file(CONFIG_FILE_NAME)?;
    Ok(config_file)
}

pub fn create_tooltip<'a>(
    element: Element<'a, Messages>,
    tooltip_element: Element<'a, Messages>,
) -> Tooltip<'a, Messages, Theme, Renderer> {
    tooltip(element, tooltip_element, tooltip::Position::FollowCursor)
        .delay(DEFAULT_TOOLTIP_DELAY)
        .style(container::bordered_box)
}
