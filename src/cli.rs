use crate::{
    common::{
        parse_executable_script, sort_by_sort_dropdown_string, Wallpaper, APP_ID, APP_VERSION,
        GETTEXT_DOMAIN,
    },
    main_window::build_ui,
    ui_common::{gschema_string_to_string, string_to_gschema_string, SORT_DROPDOWN_STRINGS},
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
};
use clap::Parser;
use gettextrs::{bind_textdomain_codeset, bindtextdomain, getters, gettext, textdomain};
use gtk::{gio::Settings, glib, prelude::*, Application};
use log::debug;
use rand::Rng;
use std::{
    env::current_exe,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use log::{error, warn};

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

fn get_previous_wallpapers(settings: &Settings) -> Vec<Wallpaper> {
    let previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(&gschema_string_to_string(
        settings.string("saved-wallpapers").as_ref(),
    ))
    .unwrap();
    previous_wallpapers
}

fn get_previous_supported_wallpapers(settings: &Settings) -> Vec<PathBuf> {
    let previous_wallpapers = get_previous_wallpapers(settings);
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
    files
}

#[must_use]
pub fn set_random_wallpapers() -> glib::ExitCode {
    let settings = Settings::new(APP_ID);
    let previous_wallpapers = get_previous_wallpapers(&settings);
    let files = get_previous_supported_wallpapers(&settings);
    WallpaperChangers::killall_changers();
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
pub fn cycle_next_wallpaper(args: &Cli) -> glib::ExitCode {
    let settings = Settings::new(APP_ID);
    let mut previous_wallpapers = get_previous_wallpapers(&settings);
    let sort_dropdown_string = SORT_DROPDOWN_STRINGS[settings.uint("sort-by") as usize];
    let mut files = get_previous_supported_wallpapers(&settings);
    let invert_sort_state = settings.boolean("invert-sort");
    sort_by_sort_dropdown_string(&mut files, sort_dropdown_string, invert_sort_state);
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
            return glib::ExitCode::FAILURE;
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
    match settings.set_string(
        "saved-wallpapers",
        &string_to_gschema_string(&serde_json::to_string(&previous_wallpapers).unwrap_or_default()),
    ) {
        Ok(_) => {}
        Err(e) => {
            error!("{} {e}", gettext("Unable to save \"next\" wallpapers"));
        }
    }
    glib::ExitCode::SUCCESS
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
        previous_wallpaper.path = path.to_str().unwrap_or_default().to_owned();
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
                previous_wallpaper.path = p.to_str().unwrap_or_default().to_owned();
            }
            None => {
                error!("Wallpaper directory is empty. Please set a wallpaper folder before using --next.");
            }
        }
    }
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
    #[arg(short, long, value_parser = parse_executable_script)]
    /// Path to external script.
    pub external_script: Option<String>,
    #[arg(long)]
    /// Set random wallpapers based on last set changer.
    pub random: bool,
    #[arg(short, long)]
    /// Get application version
    pub version: bool,
    #[arg(short, long)]
    /// Cycle wallaper(s) the next on based on the previously set wallpaper(s) and sort settings on a given monitor. "All" cycles wallpapers on all monitors.
    pub next: Option<String>,
}
