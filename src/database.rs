use crate::common::{CACHE_FILE_NAME, CONFIG_APP_NAME, CacheImageFile};
use crate::locale::TRANSLATION;
use anyhow::anyhow;
use log::{debug, trace, warn};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub struct DatabaseConnection {
    connetion: Connection,
}

impl DatabaseConnection {
    pub fn new() -> anyhow::Result<DatabaseConnection> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME);
        let cache_path = xdg_dirs.place_cache_file(CACHE_FILE_NAME)?;
        let conn = Connection::open(cache_path.to_string_lossy().as_ref())?;
        // WAL mode + a busy timeout make concurrent thumbnail cache writes from
        // the rayon thread pool much less likely to fail with "database is locked".
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 5000;
            CREATE TABLE IF NOT EXISTS imagefile
              (
                 image TEXT NOT NULL,
                 name TEXT NOT NULL,
                 date INTEGER NOT NULL,
                 path TEXT NOT NULL,
                 favorite INTEGER NOT NULL
              );
            ",
        )?;
        Ok(DatabaseConnection { connetion: conn })
    }

    pub fn select_image_file(&self, path: &Path) -> anyhow::Result<CacheImageFile> {
        let query = "SELECT image, name, date, path, favorite FROM ImageFile where path = ?1 AND typeof(image) != 'blob';";
        let mut statement = self.connetion.prepare(query)?;

        let mut pix_buf_bytes = statement
            .query_map([&path.to_string_lossy()], |row| {
                let favorite = row.get::<usize, i32>(4)?;
                let favorite = favorite > 0;
                Ok(CacheImageFile {
                    cached_image_path: PathBuf::from(row.get::<usize, String>(0)?),
                    name: row.get(1)?,
                    date: row.get(2)?,
                    path: PathBuf::from(row.get::<usize, String>(3)?),
                    favorite,
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect::<Vec<_>>();
        if pix_buf_bytes.is_empty() {
            return Err(anyhow!("No result could be found"));
        }
        let image = pix_buf_bytes.pop().unwrap();
        debug!(
            "Matches: {:#?}",
            pix_buf_bytes
                .iter()
                .filter(|i| i.favorite)
                .collect::<Vec<_>>()
        );
        debug!("Image: {image:#?}");
        Ok(image)
    }

    pub fn insert_image_file(&self, image_file: &CacheImageFile) -> anyhow::Result<()> {
        let query = "INSERT INTO ImageFile(image, name, date, path, favorite) VALUES (:image, :name, :date, :path, :favorite);";
        let favorite = i32::from(image_file.favorite);
        self.connetion.execute(
            query,
            (
                &image_file.cached_image_path.to_string_lossy(),
                &image_file.name,
                &image_file.date,
                &image_file.path.to_string_lossy(),
                &favorite,
            ),
        )?;

        Ok(())
    }

    pub fn check_cache(path: &Path) -> anyhow::Result<CacheImageFile> {
        let conn = DatabaseConnection::new()?;
        match conn.select_image_file(path) {
            Ok(mut f) => {
                // The database stores a lossy representation of the path only as a
                // cache key. Always keep the real filesystem path from the caller so
                // subsequent file operations (e.g. applying the wallpaper) work.
                f.path = path.to_path_buf();
                trace!("{}: {:#?}", TRANSLATION.get_translation("cache-hit"), f);
                Ok(f)
            }
            Err(e) => {
                trace!(
                    "{}: {} {}",
                    TRANSLATION.get_translation("cache-miss"),
                    path.to_string_lossy(),
                    e
                );
                match CacheImageFile::from_file(path) {
                    Ok(g) => {
                        trace!(
                            "{} {}",
                            TRANSLATION.get_translation("picture-created-successfully"),
                            g.path.to_string_lossy()
                        );
                        conn.insert_image_file(&g)?;
                        debug!(
                            "{} {}",
                            "Picture inserted into database.",
                            &g.path.to_string_lossy()
                        );
                        Ok(g)
                    }
                    Err(e) => {
                        warn!(
                            "{}: {} {}",
                            TRANSLATION.get_translation("file-could-not-be-converted-to-a-picture"),
                            path.to_string_lossy(),
                            e
                        );
                        Err(e)
                    }
                }
            }
        }
    }
}
