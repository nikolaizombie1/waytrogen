use crate::wallpaper_changers::{GSllapperPauseMode, WallpaperChangers};
use gettextrs::gettext;
use gtk::{
    gio::Settings, glib::clone, prelude::*, Align, Box, DropDown, Entry, Switch, TextBuffer,
};
use log::debug;
use std::{path::PathBuf, process::Command};

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
    if let WallpaperChangers::GSlapper(scale_mode, pause_mode, loop_video, additional_options) =
        gslapper_changer
    {
        debug!("gSlapper: Setting wallpaper {}", image.display());

        // Kill any existing gslapper instance first
        kill_existing_gslapper();

        // Build gslapper options
        let mut gst_options = Vec::new();

        // Add scale mode
        gst_options.push(scale_mode.to_string());

        // Add loop if enabled
        if *loop_video {
            gst_options.push("loop".to_owned());
        }

        // Always disable audio for wallpapers
        gst_options.push("no-audio".to_owned());

        // Add any additional user-provided options
        if !additional_options.is_empty() {
            gst_options.push(additional_options.clone());
        }

        let gst_options_str = gst_options.join(" ");

        let mut command = Command::new("gslapper");

        // Add IPC socket for future control
        command.arg("-I").arg(GSLAPPER_SOCKET);

        // Add pause mode
        match pause_mode {
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

pub fn generate_gslapper_changer_bar(changer_specific_options_box: &Box, settings: Settings) {
    // Scale mode dropdown
    let scale_mode_dropdown = DropDown::from_strings(&[
        &gettext("fill"),
        &gettext("stretch"),
        &gettext("original"),
        &gettext("panscan"),
    ]);
    scale_mode_dropdown.set_margin_top(12);
    scale_mode_dropdown.set_margin_start(12);
    scale_mode_dropdown.set_margin_bottom(12);
    scale_mode_dropdown.set_margin_end(12);
    scale_mode_dropdown.set_halign(Align::Start);
    scale_mode_dropdown.set_valign(Align::Center);
    scale_mode_dropdown.set_tooltip_text(Some(&gettext("Scale mode for wallpaper")));
    settings
        .bind("gslapper-scale-mode", &scale_mode_dropdown, "selected")
        .build();
    changer_specific_options_box.append(&scale_mode_dropdown);

    // Pause mode dropdown
    let pause_mode_dropdown = DropDown::from_strings(&[
        &gettext("none"),
        &gettext("auto-pause"),
        &gettext("auto-stop"),
    ]);
    pause_mode_dropdown.set_margin_top(12);
    pause_mode_dropdown.set_margin_start(12);
    pause_mode_dropdown.set_margin_bottom(12);
    pause_mode_dropdown.set_margin_end(12);
    pause_mode_dropdown.set_halign(Align::Start);
    pause_mode_dropdown.set_valign(Align::Center);
    pause_mode_dropdown.set_tooltip_text(Some(&gettext("Pause behavior when wallpaper is hidden")));
    settings
        .bind("gslapper-pause-mode", &pause_mode_dropdown, "selected")
        .build();
    changer_specific_options_box.append(&pause_mode_dropdown);

    // Loop video switch
    let loop_switch = Switch::builder()
        .tooltip_text(gettext("Loop video wallpapers"))
        .has_tooltip(true)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    settings
        .bind("gslapper-loop", &loop_switch, "active")
        .build();
    changer_specific_options_box.append(&loop_switch);

    // Additional options text entry
    let additional_options_entry = create_additional_options_textbox(&settings);
    changer_specific_options_box.append(&additional_options_entry);
}

fn create_additional_options_textbox(settings: &Settings) -> Entry {
    let additional_options = Entry::builder()
        .placeholder_text(gettext("Additional gslapper options"))
        .has_tooltip(true)
        .tooltip_text(gettext(
            "Additional GStreamer/gslapper options (e.g., panscan=0.8)",
        ))
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();

    let options_text_buffer = TextBuffer::builder().build();
    settings
        .bind("gslapper-additional-options", &options_text_buffer, "text")
        .build();

    additional_options.connect_changed(clone!(
        #[strong]
        options_text_buffer,
        move |e| {
            let text = &e.text().to_string()[..];
            options_text_buffer.set_text(text);
        }
    ));

    additional_options.set_text(
        options_text_buffer
            .text(
                &options_text_buffer.start_iter(),
                &options_text_buffer.end_iter(),
                false,
            )
            .as_str(),
    );

    additional_options
}
