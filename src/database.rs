use crate::common::CacheImageFile;
use gettextrs::gettext;
use log::{debug, trace, warn};
use sqlite::{Connection, Value};
use std::path::Path;

pub struct DatabaseConnection {
    connetion: Connection,
}

impl DatabaseConnection {
    fn new() -> anyhow::Result<DatabaseConnection> {
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

    pub fn select_image_file(&self, path: &Path) -> anyhow::Result<CacheImageFile> {
        let query = "SELECT image, name, date, path FROM GtkImageFile where path = ?;";
        let mut statement = self.connetion.prepare(query)?;

        statement.bind((1, path.to_str().unwrap()))?;
        statement.next()?;
        let pix_buf_bytes = CacheImageFile {
            image: statement.read::<Vec<u8>, _>("image")?,
            name: statement.read::<String, _>("name")?,
            date: statement.read::<i64, _>("date")? as u64,
            path: statement.read::<String, _>("path")?,
        };
        Ok(pix_buf_bytes)
    }

    pub fn insert_image_file(&self, image_file: &CacheImageFile) -> anyhow::Result<()> {
        let query =
            "INSERT INTO GtkImageFile(image, name, date, path) VALUES (:image, :name, :date, :path);";
        let mut statement = self.connetion.prepare(query)?;

        statement.bind::<&[(_, Value)]>(&[
            (":image", image_file.image.clone().into()),
            (":name", image_file.name[..].into()),
            (":date", (image_file.date as i64).into()),
            (":path", image_file.path[..].into()),
        ])?;
        statement.next()?;
        Ok(())
    }

    pub fn check_cache(path: &Path) -> Result<CacheImageFile, anyhow::Error> {
        let conn = DatabaseConnection::new()?;
        match conn.select_image_file(path) {
            Ok(f) => {
                trace!("{}: {}", gettext("Cache Hit"), f.path);
                Ok(f)
            }
            Err(e) => {
                trace!(
                    "{}: {} {}",
                    gettext("Cache Miss"),
                    path.to_str().unwrap(),
                    e
                );
                match CacheImageFile::from_file(path) {
                    Ok(g) => {
                        trace!(
                            "{} {}",
                            gettext("GTK Picture created successfully."),
                            g.path
                        );
                        conn.insert_image_file(&g)?;
                        debug!("{} {}", "Picture inserted into database.", &g.path);
                        Ok(g)
                    }
                    Err(e) => {
                        warn!(
                            "{}: {} {}",
                            gettext("File could not be converted to a GTK Picture"),
                            path.to_str().unwrap(),
                            e
                        );
                        Err(e)
                    }
                }
            }
        }
    }
}
