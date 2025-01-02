use gtk::Picture;
use image::ImageReader;
use std::{io::Cursor, path::Path, time::UNIX_EPOCH};

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
