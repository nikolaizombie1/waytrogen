use crate::wallpaper_changers::WallpaperChangers;
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
    match &sort_dropdown.to_lowercase()[..] {
        "name" => {
            files.sort_by(|f1, f2| {
                if invert_sort_switch_state {
                    f1.file_name().partial_cmp(&f2.file_name()).unwrap()
                } else {
                    f2.file_name().partial_cmp(&f1.file_name()).unwrap()
                }
            });
        }
        "date" => {
            files.sort_by(|f1, f2| {
                if invert_sort_switch_state {
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
    files
}
