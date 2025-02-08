use clap::Parser;
use waytrogen::cli::{
    launch_application, print_app_version, print_wallpaper_state, restore_wallpapers,
    set_random_wallpapers, Cli,
};

use gtk::glib;

fn main() -> glib::ExitCode {
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level as usize)
        .init()
        .unwrap();
    // Create a new application

    if args.restore {
        restore_wallpapers()
    } else if args.list_current_wallpapers {
        print_wallpaper_state()
    } else if args.random {
        set_random_wallpapers()
    } else if args.version {
        print_app_version()
    } else {
        launch_application(args)
    }
}
