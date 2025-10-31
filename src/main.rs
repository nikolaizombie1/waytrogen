use clap::Parser;
use gtk::glib;
use log::error;
use std::{thread::sleep, time::Duration};
use waytrogen::{
    cli::{
        cycle_next_wallpaper, delete_image_cache, launch_application, print_app_version,
        print_wallpaper_state, restore_wallpapers, set_random_wallpapers, Cli,
    },
    dotfile::{self, get_config_file},
};

fn main() -> glib::ExitCode {
    let mut args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level as usize)
        .init()
        .unwrap();

    let config_file = match get_config_file() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get config file: {e}");
            return glib::ExitCode::FAILURE;
        }
    };

    match config_file.write_to_gsettings() {
	Ok(_) => {},
	Err(e) => {
	    error!("Failed to write gsettings from configuration file: {e}");
	    return glib::ExitCode::FAILURE;
	}
    }

    if args.external_script.is_none() && !config_file.executable_script.is_empty() {
        args.external_script = Some(config_file.executable_script);
    }

    if args.restore {
        sleep(Duration::from_millis(args.startup_delay));
        restore_wallpapers()
    } else if args.list_current_wallpapers {
        print_wallpaper_state()
    } else if args.random {
        sleep(Duration::from_millis(args.startup_delay));
        set_random_wallpapers()
    } else if args.version {
        print_app_version()
    } else if args.next.is_some() {
        sleep(Duration::from_millis(args.startup_delay));
        cycle_next_wallpaper(&args)
    } else if args.delete_cache {
        delete_image_cache()
    } else {
        let _ = launch_application(args);

        let config_file = match dotfile::ConfigFile::from_gsettings() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get config file: {e}");
                return glib::ExitCode::FAILURE;
            }
        };
        match config_file.write_to_config_file() {
            Ok(_) => glib::ExitCode::SUCCESS,
            Err(_) => glib::ExitCode::FAILURE,
        }
    }
}
