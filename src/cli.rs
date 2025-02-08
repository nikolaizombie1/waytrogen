use crate::{
    common::{Wallpaper, APP_ID, APP_VERSION, GETTEXT_DOMAIN},
    main_window::build_ui,
    ui_common::gschema_string_to_string,
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
};
use clap::Parser;
use gettextrs::{bind_textdomain_codeset, bindtextdomain, getters, textdomain};
use gtk::{gio::Settings, glib, prelude::*, Application};
use log::debug;
use rand::Rng;
use std::{
    env::current_exe,
    fs::File,
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

#[must_use]
pub fn restore_wallpapers() -> glib::ExitCode {
    let settings = Settings::new(APP_ID);
    WallpaperChangers::killall_changers();
    let previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(&gschema_string_to_string(
        settings.string("saved-wallpapers").as_ref(),
    ))
    .unwrap();
    for wallpaper in previous_wallpapers {
        debug!("Restoring: {:?}", wallpaper);
        wallpaper.clone().changer.change(
            PathBuf::from(wallpaper.clone().path),
            wallpaper.clone().monitor,
        );
        match wallpaper.clone().changer {
            WallpaperChangers::Hyprpaper => {
                thread::sleep(Duration::from_millis(1000));
            }
            WallpaperChangers::Swaybg(_, _)
            | WallpaperChangers::MpvPaper(_, _, _)
            | WallpaperChangers::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {}
        }
    }
    glib::ExitCode::SUCCESS
}

#[must_use]
pub fn print_wallpaper_state() -> glib::ExitCode {
    let settings = Settings::new(APP_ID);
    println!(
        "{}",
        gschema_string_to_string(&settings.string("saved-wallpapers"))
    );
    glib::ExitCode::SUCCESS
}

pub fn set_random_wallpapers() -> glib::ExitCode {
    let settings = Settings::new(APP_ID);
    WallpaperChangers::killall_changers();
    let previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(&gschema_string_to_string(
        settings.string("saved-wallpapers").as_ref(),
    ))
    .unwrap();
    let wallpaper = previous_wallpapers[0].clone();
    let path = Path::new(&wallpaper.path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let files = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .map(|d| d.path().to_path_buf())
        .filter(|p| {
            previous_wallpapers
                .iter()
                .map(|w| w.changer.clone())
                .all(|c| {
                    c.accepted_formats().iter().any(|f| {
                        f == p
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    })
                })
        })
        .collect::<Vec<_>>();
    for w in &previous_wallpapers {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..files.len());
        log::debug!("{index}");
        w.changer
            .clone()
            .change(files[index].clone(), w.monitor.clone());
    }
    glib::ExitCode::SUCCESS
}

#[must_use]
pub fn print_app_version() -> glib::ExitCode {
    println!("{APP_VERSION}");
    glib::ExitCode::SUCCESS
}

#[must_use]
pub fn launch_application(args: Cli) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    textdomain("waytrogen").unwrap();
    bind_textdomain_codeset("waytrogen", "UTF-8").unwrap();
    let os_id = get_os_id().unwrap().unwrap_or_default();
    let domain_directory = match os_id.as_str() {
        "nixos" => {
            #[cfg(feature = "nixos")]
            // the path is known at compile time when using nix to build waytrogen
            {
                let path = env!("OUT_PATH").parse::<PathBuf>().unwrap();
                path.join("share").join("locale")
            }

            #[cfg(not(feature = "nixos"))]
            {
                let exe_path = current_exe().unwrap();
                exe_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("share")
                    .join("locale")
            }
        }
        _ => getters::domain_directory(GETTEXT_DOMAIN).unwrap(),
    };
    bindtextdomain(GETTEXT_DOMAIN, domain_directory).unwrap();

    app.connect_activate(move |app| {
        build_ui(app, &args);
    });

    let empty: Vec<String> = vec![];
    // Run the application
    app.run_with_args(&empty)
}

/// os id is the ID="nixos" parameter in `/etc/os-release`
/// If ID parameter is not found this returns None
fn get_os_id() -> anyhow::Result<Option<String>> {
    let file = File::open("/etc/os-release")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Some(s) = line.strip_prefix("ID=") {
            let id = s.trim_matches('"');
            return Ok(Some(id.to_string()));
        }
    }
    Ok(None)
}

#[derive(Parser, Clone)]
pub struct Cli {
    #[arg(short, long)]
    /// Restore previously set wallpapers
    pub restore: bool,
    #[arg(short, long, default_value_t = 0)]
    /// How many error, warning, info, debug or trace logs will be shown. 0 for error, 1 for warning, 2 for info, 3 for debug, 4 or higher for trace.
    pub log_level: u8,
    #[arg(short, long, default_value_t = false)]
    /// Get the current wallpaper settings in JSON format.
    pub list_current_wallpapers: bool,
    #[arg(short, long, value_parser = parse_executable_script, default_value_t = String::from(""))]
    /// Path to external script.
    pub external_script: String,
    #[arg(long)]
    /// Set random wallpapers based on last set changer.
    pub random: bool,
    #[arg(short, long)]
    /// Get application version
    pub version: bool,
}

fn parse_executable_script(s: &str) -> anyhow::Result<String> {
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
