use crate::{common::sort_by_sort_dropdown_string, wallpaper_changers::WallpaperChangers};
use std::path::PathBuf;
#[must_use]
pub fn get_image_files(
    path: &str,
    sort_dropdown: &str,
    invert_sort_switch_state: bool,
) -> Vec<PathBuf> {
    let mut files = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .map(|d| d.path().to_path_buf())
        .filter(|p| {
            WallpaperChangers::all_accepted_formats().iter().any(|f| {
                f == p
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();
    sort_by_sort_dropdown_string(&mut files, sort_dropdown, invert_sort_switch_state);
    files
}
