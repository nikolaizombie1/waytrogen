use crate::{
    app_state::{self, AppState, Messages},
    common::DEFAULT_MARGIN,
    wallpaper_changers::{GSllapperPauseMode, GSllapperScaleMode, WallpaperChangers},
};
use gettextrs::gettext;
use iced::{
    Alignment::Center,
    Element,
    widget::{pick_list, row, text_input, toggler},
};
use log::debug;
use std::{path::PathBuf, process::Command};
use strum::VariantArray;

const GSLAPPER_SOCKET: &str = "/tmp/gslapper.sock";

/// Kill any existing gslapper instance before starting a new one
fn kill_existing_gslapper() {
    let socket_path = std::path::Path::new(GSLAPPER_SOCKET);

    // Try graceful quit via IPC first (using socat if available)
    if socket_path.exists() {
        debug!("gSlapper: Attempting graceful quit via IPC");
        // Try socat first (more commonly available than nc on some systems)
        let ipc_result = Command::new("bash")
            .arg("-c")
            .arg(format!("echo quit | socat - UNIX-CONNECT:{GSLAPPER_SOCKET} 2>/dev/null || echo quit | nc -U {GSLAPPER_SOCKET} 2>/dev/null"))
            .spawn()
            .and_then(|mut c| c.wait());

        if ipc_result.is_ok() {
            // Give it a moment to quit gracefully
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    // Always use pkill as fallback/confirmation to ensure process is dead
    debug!("gSlapper: Ensuring process is killed with pkill");
    let _ = Command::new("pkill")
        .arg("-9")
        .arg("gslapper")
        .spawn()
        .and_then(|mut c| c.wait());

    // Small delay to ensure process is fully terminated
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Clean up socket file if it still exists
    if socket_path.exists() {
        let _ = std::fs::remove_file(GSLAPPER_SOCKET);
    }
}

pub fn change_gslapper_wallpaper(
    gslapper_changer: &WallpaperChangers,
    image: PathBuf,
    monitor: &str,
) {
    if let WallpaperChangers::GSlapper(settings) = gslapper_changer {
        debug!("gSlapper: Setting wallpaper {}", image.display());

        // Kill any existing gslapper instance first
        kill_existing_gslapper();

        // Build gslapper options
        let mut gst_options = Vec::new();

        // Add scale mode
        gst_options.push(settings.scale_mode.to_string());

        // Add loop if enabled
        if settings.loop_video {
            gst_options.push("loop".to_owned());
        }

        // Always disable audio for wallpapers
        gst_options.push("no-audio".to_owned());

        // Add any additional user-provided options
        if !settings.additional_options.is_empty() {
            gst_options.push(settings.additional_options.clone());
        }

        let gst_options_str = gst_options.join(" ");

        let mut command = Command::new("gslapper");

        // Add IPC socket for future control
        command.arg("-I").arg(GSLAPPER_SOCKET);

        // Add pause mode
        match settings.pause_mode {
            GSllapperPauseMode::None => {}
            GSllapperPauseMode::AutoPause => {
                command.arg("-p");
            }
            GSllapperPauseMode::AutoStop => {
                command.arg("-s");
            }
        }

        // Add GStreamer options
        command.arg("-o").arg(&gst_options_str);

        // Fork to background
        command.arg("-f");

        // Add monitor (use '*' for all monitors)
        let monitor_arg = if monitor == gettext("All") {
            "*"
        } else {
            monitor
        };
        command.arg(monitor_arg);

        // Add the wallpaper path
        command.arg(&image);

        debug!("gSlapper: Running command: {:?}", command);

        command.spawn().unwrap().wait().unwrap();
    }
}

pub fn generate_gslapper_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let scale_mode_dropdown: Element<'_, Messages> = pick_list(
        GSllapperScaleMode::VARIANTS,
        app_state.gslapper_scale_mode,
        Messages::GSllaperScaleModeChanged,
    ).into();

    let pause_mode_dropdown: Element<'_, Messages> = pick_list(
        GSllapperPauseMode::VARIANTS,
        app_state.gslapper_pause_mode,
        Messages::GSlapperPauseModeChanged,
    ).into();

    let loop_switch: Element<'_, Messages> =
        toggler(app_state.gslapper_loop).on_toggle(Messages::GSllaperLoopVideoChanged).into();

    let additional_options_entry: Element<'_, Messages> = text_input(
        &gettext("Additional GStreamer/gslapper options (e.g., panscan=0.8)"),
        &app_state.gslapper_additional_options,
    )
    .on_input(Messages::GSllaperAdditionalOptionsChanged).into();

    vec![
        scale_mode_dropdown,
        pause_mode_dropdown,
        loop_switch,
        additional_options_entry
    ]
}
