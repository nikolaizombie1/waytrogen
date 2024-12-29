use anyhow::Ok;
use gtk::{gdk::Texture, glib::Bytes, Picture};
use image::ImageReader;
use std::{io::Cursor, path::Path, time::UNIX_EPOCH};

pub const THUMBNAIL_HEIGHT: i32 = 200;
pub const THUMBNAIL_WIDTH: i32 = THUMBNAIL_HEIGHT;

#[derive(Clone)]
pub struct GtkImageFile {
    pub image: Picture,
    pub name: String,
    pub date: u64,
    pub path: String,
}

impl GtkImageFile {
    pub fn from_file(path: &Path) -> anyhow::Result<GtkImageFile> {
        let image = Self::generate_thumbnail(path)?;
        image.set_can_shrink(true);
        Self::create_gtk_image(path, image)
    }

    fn get_metadata(path: &Path) -> anyhow::Result<(String, String, u64)> {
        let path = path.to_path_buf();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let date = std::fs::File::open(path.clone())?.metadata()?.created()?;
        let date = date.duration_since(UNIX_EPOCH)?.as_secs();
        Ok((path.to_str().unwrap().to_string(), name, date))
    }

    fn create_gtk_image(path: &Path, image: Picture) -> anyhow::Result<GtkImageFile> {
        let fields = Self::get_metadata(path)?;
        let image_file = GtkImageFile {
            image,
            path: fields.0,
            name: fields.1,
            date: fields.2,
        };
        Ok(image_file)
    }

    fn generate_thumbnail(path: &Path) -> anyhow::Result<Picture> {
        let thumbnail = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?
            .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
            .to_rgb8();
        let mut buff: Vec<u8> = vec![];
        thumbnail.write_to(&mut Cursor::new(&mut buff), image::ImageFormat::Png)?;
        let picture = Picture::for_paintable(&Texture::from_bytes(&Bytes::from(&buff))?);
        Ok(picture)
    }
}
