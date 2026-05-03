use crate::{
    app_state::{self, AppState, Messages},
    common::DEFAULT_MARGIN,
    wallpaper_changers::{
        MpvPaperPauseModes, MpvPaperSettings, MpvPaperSlideshowSettings, WallpaperChanger,
        WallpaperChangers,
    },
};
use gettextrs::gettext;
use iced::{
    Alignment::Center,
    Element,
    widget::{pick_list, row, text_input, toggler},
};
use iced_aw::number_input;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use strum::VariantArray;

const ALL_MONITOR_SOCKET: &str = "/tmp/mpv-socket-All";

pub fn change_mpvpaper_wallpaper(
    mpvpaper_changer: &WallpaperChangers,
    image: PathBuf,
    monitor: &str,
) {
    if let WallpaperChangers::MpvPaper(settings) = mpvpaper_changer {
        log::debug!("{}", image.display());
        let mut command = Command::new("mpvpaper");
        let socket = if monitor == gettext("All") {
            String::from(ALL_MONITOR_SOCKET)
        } else {
            format!("/tmp/mpv-socket-{monitor}")
        };

        let mpv_options = format!("input-ipc-server={socket} {}", settings.additional_options);

        let monitor = if monitor == gettext("All") {
            "*"
        } else {
            monitor
        };
        command.arg("-o").arg(mpv_options);
        match settings.pause_mode {
            MpvPaperPauseModes::None => {}
            MpvPaperPauseModes::AutoPause => {
                command.arg("--auto-pause");
            }
            MpvPaperPauseModes::AutoStop => {
                command.arg("--auto-stop");
            }
        }
        if settings.slideshow_settings.enable {
            command
                .arg("-n")
                .arg(settings.slideshow_settings.seconds.to_string());
        }

        let socket_path = std::path::Path::new(&socket);

        if socket_path.exists() {
            log::debug!("Attempting to close socket.");
            Command::new("bash")
                .arg("-c")
                .arg(format!("echo quit | socat - {socket}"))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            Command::new("rm")
                .arg(socket)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        let all_monitor_socket_exists = std::path::Path::new(ALL_MONITOR_SOCKET).exists();

        if all_monitor_socket_exists && monitor != gettext("All") {
            Command::new("bash")
                .arg("-c")
                .arg(format!("echo quit | socat - {ALL_MONITOR_SOCKET}"))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        } else if all_monitor_socket_exists && monitor == gettext("All") {
            mpvpaper_changer.kill();
        }

        command
            .arg(monitor)
            .arg(image)
            .arg("-f")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_mpvpaper_changer_bar(app_state: &AppState) -> Element<'_, Messages> {
    let pause_options_dropdown = pick_list(
        MpvPaperPauseModes::VARIANTS,
        app_state.mpvpaper_pause_option.clone(),
        Messages::MpvPaperPauseModeChanged,
    );
    let slideshow_enable_switch = toggler(app_state.mpvpaper_slideshow_enable)
        .on_toggle(Messages::MpvPaperEnableSlideshowChanged);
    let slidehow_interval_input = number_input(
        &app_state.mpvpaper_slideshow_interval,
        0..,
        Messages::MpvPaperSlideshowIntervalChanged,
    );
    let mpv_options = text_input(
        &gettext("Additional MPV Options"),
        &app_state.mpvpaper_additional_options,
    )
    .on_input(Messages::MpvPaperAdditionalOptionsChanged);

    row![
        pause_options_dropdown,
        slideshow_enable_switch,
        slidehow_interval_input,
        mpv_options
    ]
    .align_y(Center)
    .spacing(DEFAULT_MARGIN as f32)
    .into()
}
