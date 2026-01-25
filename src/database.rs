use crate::{cli::delete_image_cache, common::{CACHE_FILE_NAME, CONFIG_APP_NAME, CacheImageFile}};
use anyhow::anyhow;
use gettextrs::gettext;
use log::{debug, trace, warn};
use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};

pub struct DatabaseConnection {
    connetion: Connection,
}

impl DatabaseConnection {
    fn new() -> anyhow::Result<DatabaseConnection> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME)?;
        let cache_path = xdg_dirs.place_cache_file(CACHE_FILE_NAME)?;
        let conn = Connection::open(cache_path.to_str().unwrap())?;
        let query = "
      CREATE TABLE IF NOT EXISTS gtkimagefile
        (
           image TEXT NOT NULL,
           name TEXT NOT NULL,
           date INTEGER NOT NULL,
           path TEXT NOT NULL
        );
     ";
        conn.execute(query, ())?;
        Ok(DatabaseConnection { connetion: conn })
    }

    pub fn select_image_file(&self, path: &Path) -> anyhow::Result<CacheImageFile> {
        let query = "SELECT image, name, date, path FROM GtkImageFile where path = ?1 AND typeof(image) != 'blob';";
        let mut statement = self.connetion.prepare(query)?;

        let pix_buf_bytes = statement
            .query_map([path.to_str().unwrap_or_default()], |row| {
                Ok(CacheImageFile {
                    cached_image_path: PathBuf::from(row.get::<usize, String>(0)?),
                    name: row.get(1)?,
                    date: row.get(2)?,
                    path: row.get(3)?,
                })
            })?
            .filter_map(|c| c.ok())
            .collect::<Vec<_>>();
        if pix_buf_bytes.is_empty() {
            return Err(anyhow!("No result could be found"));
        }
	let image = pix_buf_bytes[0].clone();
        Ok(image)
    }

    pub fn insert_image_file(&self, image_file: &CacheImageFile) -> anyhow::Result<()> {
        let query =
            "INSERT INTO GtkImageFile(image, name, date, path) VALUES (:image, :name, :date, :path);";
        self.connetion.execute(
            query,
            (
                &image_file.cached_image_path.to_str().unwrap(),
                &image_file.name,
                &image_file.date,
                &image_file.path,
            ),
        )?;

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
