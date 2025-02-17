use clap::Parser;
use log::error;
use waytrogen::{
    cli::{
        cycle_next_wallpaper, launch_application, print_app_version, print_wallpaper_state,
        restore_wallpapers, set_random_wallpapers, Cli,
    },
    dotfile::get_config_file,
};

use gtk::glib;

fn main() -> glib::ExitCode {
    let mut args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level as usize)
        .init()
        .unwrap();

    let config_file = get_config_file();
    if config_file.is_err() {
        error!(
            "Failed to get config file: {}",
            config_file.as_ref().err().unwrap()
        );
        return glib::ExitCode::FAILURE;
    }
    let config_file = config_file.unwrap();

    if args.external_script.is_none() && !config_file.executable_script.is_empty() {
        args.external_script = Some(config_file.executable_script);
    }

    if args.restore {
        restore_wallpapers()
    } else if args.list_current_wallpapers {
        print_wallpaper_state()
    } else if args.random {
        set_random_wallpapers()
    } else if args.version {
        print_app_version()
    } else if args.next.is_some() {
        cycle_next_wallpaper(&args)
    } else {
        launch_application(args)
    }
}
