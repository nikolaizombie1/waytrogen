use crate::{
    app_state::{AppState, Messages},
    locale::TRANSLATION,
    wallpaper_changers::{MpvPaperPauseModes, MpvPaperSettings, WallpaperChangers},
};
use iced::{
    Element,
    widget::{pick_list, text_input, toggler},
};
use iced_aw::number_input;
use std::default::Default;
use std::sync::LazyLock;
use std::{path::PathBuf, process::Command, sync::Mutex};
use strum::VariantArray;

static SPAWNED_MPVPAPER_PROCESSES: LazyLock<Mutex<Vec<MpvPaperWallpaper>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Default, Debug)]
struct MpvPaperWallpaper {
    pub settings: MpvPaperSettings,
    pub image: PathBuf,
    pub monitor: String,
}

pub fn change_mpvpaper_wallpaper(
    mpvpaper_changer: &WallpaperChangers,
    image: PathBuf,
    monitor: &str,
) {
    if let WallpaperChangers::MpvPaper(settings) = mpvpaper_changer {
        // Acquire once, hold for all operations
        let mut previous_wallpapers = SPAWNED_MPVPAPER_PROCESSES.lock().unwrap();

        Command::new("pkill")
            .arg("-9")
            .arg("mpvpaper")
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();

        // Kill existing process on this monitor
        if monitor == TRANSLATION.get_translation("All") {
            previous_wallpapers.retain(|_| false);
        } else {
            previous_wallpapers.retain(|m| m.monitor != TRANSLATION.get_translation("All"))
        }

        if let Some(w) = previous_wallpapers
            .iter_mut()
            .find(|m| m.monitor == monitor)
        {
            w.image.clone_from(&image);
        } else {
            previous_wallpapers.push(MpvPaperWallpaper {
                settings: settings.clone(),
                image: image.clone(),
                monitor: monitor.to_string(),
            });
        }

        for wallpaper in previous_wallpapers.iter() {
            let mut command = Command::new("mpvpaper");
            let mpv_options = format!("{}", wallpaper.settings.additional_options);
            let monitor = if wallpaper.monitor == TRANSLATION.get_translation("All") {
                "*"
            } else {
                &wallpaper.monitor
            };

            command.arg("-o").arg(mpv_options);
            match wallpaper.settings.pause_mode {
                MpvPaperPauseModes::None => {}
                MpvPaperPauseModes::AutoPause => {
                    command.arg("--auto-pause");
                }
                MpvPaperPauseModes::AutoStop => {
                    command.arg("--auto-stop");
                }
            }
            if wallpaper.settings.slideshow_settings.enable {
                command
                    .arg("-n")
                    .arg(wallpaper.settings.slideshow_settings.seconds.to_string());
            }

            command
                .arg(monitor)
                .arg(wallpaper.image.clone())
                .arg("-f")
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
        }

        // Lock is released here when `processes` drops
    }
}

pub fn generate_mpvpaper_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let pause_options_dropdown: Element<'_, Messages> = pick_list(
        MpvPaperPauseModes::VARIANTS,
        app_state.mpvpaper_pause_option,
        Messages::MpvPaperPauseModeChanged,
    )
    .into();

    let slideshow_enable_switch: Element<'_, Messages> =
        toggler(app_state.mpvpaper_slideshow_enable)
            .on_toggle(Messages::MpvPaperEnableSlideshowChanged)
            .into();

    let slidehow_interval_input: Element<'_, Messages> = number_input(
        &app_state.mpvpaper_slideshow_interval,
        0..,
        Messages::MpvPaperSlideshowIntervalChanged,
    )
    .into();

    let mpv_options: Element<'_, Messages> = text_input(
        &TRANSLATION.get_translation("additional-mpv-options"),
        &app_state.mpvpaper_additional_options,
    )
    .on_input(Messages::MpvPaperAdditionalOptionsChanged)
    .into();

    vec![
        pause_options_dropdown,
        slideshow_enable_switch,
        slidehow_interval_input,
        mpv_options,
    ]
}
