use anyhow::anyhow;
use clap::Parser;
use log::error;
use std::{thread::sleep, time::Duration};
use waytrogen::{
    app_state::AppState,
    cli::{
        Cli, cycle_next_wallpaper, delete_image_cache, print_app_version, print_wallpaper_state,
        restore_wallpapers, set_random_wallpapers,
    },
    dotfile::get_config_file,
};

fn main() -> anyhow::Result<()> {
    let mut args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level as usize)
        .init()
        .unwrap();

    let mut config_file: AppState = match get_config_file() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get config file: {e}");
            return Err(e.into());
        }
    };

    if args.external_script.is_none() && !config_file.executable_script.is_empty() {
        args.external_script = Some(config_file.executable_script.clone());
    }

    if args.restore {
        sleep(Duration::from_millis(args.startup_delay));
        restore_wallpapers(&config_file)
    } else if args.list_current_wallpapers {
        print_wallpaper_state(&config_file)
    } else if args.random {
        sleep(Duration::from_millis(args.startup_delay));
        match set_random_wallpapers(&mut config_file) {
            Ok(_) => config_file.write_to_config_file(),
            Err(e) => Err(e),
        }
    } else if args.version {
        print_app_version()
    } else if args.next.is_some() {
        sleep(Duration::from_millis(args.startup_delay));
        match cycle_next_wallpaper(&args, &mut config_file) {
            Ok(_) => config_file.write_to_config_file(),
            Err(e) => Err(e),
        }
    } else if args.delete_cache {
        delete_image_cache()
    } else {
        match AppState::run_application(config_file) {
            Ok(_) => Ok(()),
            Err(e) => return Err(anyhow!("{e}")),
        }
    }
}
