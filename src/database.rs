use crate::common::GtkImageFile;
use anyhow::Ok;
use gtk::{
    gdk::Texture,
    glib::Bytes,
    prelude::{Cast, TextureExt},
    Picture,
};
use log::trace;
use sqlite::{Connection, Value};
use std::path::Path;

pub struct DatabaseConnection {
    connetion: Connection,
}

impl DatabaseConnection {
    pub fn new() -> anyhow::Result<DatabaseConnection> {

        let xdg_dirs = xdg::BaseDirectories::with_prefix("waytrogen")?;
        let cache_path = xdg_dirs.place_cache_file("cache.db")?;
        let conn = sqlite::open(cache_path.to_str().unwrap())?;
        let query = "
      CREATE TABLE IF NOT EXISTS gtkimagefile
        (
           image TEXT NOT NULL,
           name TEXT NOT NULL,
           date INTEGER NOT NULL,
           path TEXT NOT NULL
        );
     ";
        conn.execute(query)?;
        Ok(DatabaseConnection { connetion: conn })
    }

    pub fn select_image_file(&self, path: &Path) -> anyhow::Result<GtkImageFile> {
        let query = "SELECT image, name, date, path FROM GtkImageFile where path = ?;";
        let mut statement = self.connetion.prepare(query)?;

        statement.bind((1, path.to_str().unwrap()))?;
        statement.next()?;
        let pix_buf_bytes = GtkImageFile {
            image: Picture::for_paintable(&Texture::from_bytes(&Bytes::from(
                &statement.read::<Vec<u8>, _>("image")?,
            ))?),
            name: statement.read::<String, _>("name")?,
            date: statement.read::<i64, _>("date")? as u64,
            path: statement.read::<String, _>("path")?,
        };
        Ok(pix_buf_bytes)
    }

    pub fn insert_image_file(&self, image_file: &GtkImageFile) -> anyhow::Result<()> {
        let query =
            "INSERT INTO GtkImageFile(image, name, date, path) VALUES (:image, :name, :date, :path);";
        let mut statement = self.connetion.prepare(query)?;

        statement.bind::<&[(_, Value)]>(&[
            (
                ":image",
                image_file
                    .image
                    .paintable()
                    .unwrap()
                    .downcast::<Texture>()
                    .unwrap()
                    .save_to_png_bytes()
                    .to_vec()
                    .as_slice()
                    .into(),
            ),
            (":name", image_file.name[..].into()),
            (":date", (image_file.date as i64).into()),
            (":path", image_file.path[..].into()),
        ])?;
        trace!("Statement Bound Correctly.");
        statement.next()?;
        Ok(())
    }
}
