use crate::{
    app_state::AppState,
    common::{
        APP_VERSION, CACHE_FILE_NAME, CONFIG_APP_NAME, Wallpaper,
        sort_by_sort_dropdown_string,
    },
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
    cli_parser::Cli
};
use anyhow::anyhow;
use log::debug;
use std::{
    fs::remove_dir_all,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use log::{error, warn};

pub fn restore_wallpapers(app_state: &AppState) -> anyhow::Result<()> {
    WallpaperChangers::killall_changers();
    let previous_wallpapers = app_state.saved_wallpapers.clone();
    for wallpaper in previous_wallpapers {
        if wallpaper == Wallpaper::default() {
            continue;
        }
        debug!("Restoring: {wallpaper:?}");
        wallpaper.clone().changer.change(
            PathBuf::from(wallpaper.clone().path),
            wallpaper.clone().monitor,
        );
        match wallpaper.clone().changer {
            WallpaperChangers::Hyprpaper(_) => {
                thread::sleep(Duration::from_secs(1));
            }
            WallpaperChangers::Swaybg(_)
            | WallpaperChangers::MpvPaper(_)
            | WallpaperChangers::Awww(_)
            | WallpaperChangers::GSlapper(_) => {}
        }
    }
    Ok(())
}

pub fn print_wallpaper_state(app_state: &AppState) -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&app_state.saved_wallpapers)?
    );
    Ok(())
}

fn get_previous_supported_wallpapers(app_state: &AppState) -> Vec<PathBuf> {
    let previous_wallpapers = app_state.saved_wallpapers.clone();
    let wallpaper = previous_wallpapers[0].clone();
    let path = Path::new(&wallpaper.path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    walkdir::WalkDir::new(path)
        .follow_links(true)
        .follow_root_links(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .map(|d| d.path().to_path_buf())
        .filter(|p| {
            previous_wallpapers
                .iter()
                .map(|w| w.changer.clone())
                .all(|c: WallpaperChangers| {
                    c.accepted_formats().iter().any(|f| {
                        f == p
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    })
                })
        })
        .collect::<Vec<_>>()
}

pub fn set_random_wallpapers(app_state: &mut AppState) -> anyhow::Result<()> {
    let mut previous_wallpapers = app_state.saved_wallpapers.clone();
    let files = get_previous_supported_wallpapers(app_state);
    WallpaperChangers::killall_changers();
    for w in &mut previous_wallpapers {
        let index = rand::random_range(0..files.len());
        log::debug!("{index}");
        w.changer
            .clone()
            .change(files[index].clone(), w.monitor.clone());
        files[index]
            .clone()
            .to_str()
            .unwrap_or_default()
            .clone_into(&mut w.path);
    }
    app_state.saved_wallpapers = previous_wallpapers;
    Ok(())
}

pub fn print_app_version() -> anyhow::Result<()> {
    println!("{APP_VERSION}");
    Ok(())
}

pub fn cycle_next_wallpaper(args: &Cli, app_state: &mut AppState) -> anyhow::Result<()> {
    let mut previous_wallpapers = app_state.saved_wallpapers.clone();
    let sort_dropdown_string = app_state.sort_by.clone().unwrap_or_default();
    let mut files = get_previous_supported_wallpapers(app_state);
    let invert_sort_state = app_state.invert_sort;
    sort_by_sort_dropdown_string(&mut files, &sort_dropdown_string, invert_sort_state);
    if args.next.clone().unwrap_or_default() == "All" {
        for previous_wallpaper in &mut previous_wallpapers {
            let wallpaper_index = files.iter().position(|p| {
                p.clone()
                    == previous_wallpaper
                        .path
                        .parse::<PathBuf>()
                        .unwrap_or_default()
            });
            try_set_next_wallpaper(&files, wallpaper_index, previous_wallpaper);
        }
    } else {
        let previous_wallpaper = previous_wallpapers
            .iter()
            .find(|w| *w.monitor == args.next.clone().unwrap_or_default());
        if previous_wallpaper.is_none() {
            error!(
                "Display \"{}\" does not exist.",
                args.next.clone().unwrap_or_default()
            );
            return Err(anyhow!("Failed to get previous wallpaper"));
        }
        let mut previous_wallpaper = previous_wallpaper.unwrap().clone();
        try_set_next_wallpaper(
            &files,
            files.iter().position(|f| {
                *f == previous_wallpaper
                    .path
                    .parse::<PathBuf>()
                    .unwrap_or_default()
            }),
            &mut previous_wallpaper,
        );
        let index = previous_wallpapers
            .iter()
            .position(|w| w.monitor == previous_wallpaper.monitor)
            .unwrap();
        previous_wallpapers[index] = previous_wallpaper;
    }
    app_state.saved_wallpapers = previous_wallpapers;
    Ok(())
}

fn try_set_next_wallpaper(
    files: &[PathBuf],
    position: Option<usize>,
    previous_wallpaper: &mut Wallpaper,
) {
    if let Some(i) = position {
        let path = &files[(i + 1) % files.len()];
        previous_wallpaper
            .changer
            .clone()
            .change(path.clone(), previous_wallpaper.monitor.clone());
        path.to_str()
            .unwrap_or_default()
            .clone_into(&mut previous_wallpaper.path);
    } else {
        warn!(
            "Wallpaper {} could not be found. Using first wallpaper",
            previous_wallpaper
                .path
                .parse::<PathBuf>()
                .unwrap_or_default()
                .display()
        );
        match files.first() {
            Some(p) => {
                previous_wallpaper
                    .changer
                    .clone()
                    .change(p.clone(), previous_wallpaper.monitor.clone());
                p.to_str()
                    .unwrap_or_default()
                    .clone_into(&mut previous_wallpaper.path);
            }
            None => {
                error!(
                    "Wallpaper directory is empty. Please set a wallpaper folder before using --next."
                );
            }
        }
    }
}

pub fn delete_image_cache() -> anyhow::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(CONFIG_APP_NAME);
    let cache_path = xdg_dirs.place_cache_file(CACHE_FILE_NAME);
    if cache_path.is_err() {
        let msg = format!("Failed to get cache path, {}", cache_path.err().unwrap());
        error!("{msg}");
        return Err(anyhow!("{msg}"));
    }

    let Some(cache_home_dir) = xdg_dirs.get_config_home() else {
        return Err(anyhow!("Failed to get XDG config home"));
    };

    let Some(cache_dir) = xdg_dirs.get_cache_home() else {
        return Err(anyhow!("Failed to get XDG cache home"));
    };

    match remove_dir_all(cache_dir) {
        Ok(()) => {}
        Err(e) => {
            let msg = format!("Failed to delete cache: {e}");
            error!("{msg}");
            return Err(anyhow!("{msg}"));
        }
    }

    match remove_dir_all(cache_home_dir) {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = format!("Failed to delete config file: {e}");
            error!("{msg}");
            Err(anyhow!("{msg}"))
        }
    }
}

