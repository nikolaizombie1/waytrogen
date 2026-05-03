use crate::{
    app_state::{AppState, Messages},
    wallpaper_changers::{
        MpvPaperPauseModes, WallpaperChanger,
        WallpaperChangers,
    },
};
use gettextrs::gettext;
use iced::{
    Element, widget::{pick_list, text_input, toggler}
};
use iced_aw::number_input;
use std::{
    path::PathBuf,
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

pub fn generate_mpvpaper_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let pause_options_dropdown: Element<'_, Messages> = pick_list(
        MpvPaperPauseModes::VARIANTS,
        app_state.mpvpaper_pause_option,
        Messages::MpvPaperPauseModeChanged,
    ).into();

    let slideshow_enable_switch: Element<'_, Messages> = toggler(app_state.mpvpaper_slideshow_enable)
        .on_toggle(Messages::MpvPaperEnableSlideshowChanged).into();

    let slidehow_interval_input: Element<'_, Messages> = number_input(
        &app_state.mpvpaper_slideshow_interval,
        0..,
        Messages::MpvPaperSlideshowIntervalChanged,
    ).into();

    let mpv_options: Element<'_, Messages> = text_input(
        &gettext("Additional MPV Options"),
        &app_state.mpvpaper_additional_options,
    )
    .on_input(Messages::MpvPaperAdditionalOptionsChanged).into();

    vec![
        pause_options_dropdown,
        slideshow_enable_switch,
        slidehow_interval_input,
        mpv_options
    ]
}
