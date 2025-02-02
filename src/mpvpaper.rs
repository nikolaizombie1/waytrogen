use crate::wallpaper_changers::{
    MpvPaperPauseModes, MpvPaperSlideshowSettings, WallpaperChanger, WallpaperChangers,
};
use gettextrs::gettext;
use gtk::{
    gio::Settings, glib::clone, prelude::*, Adjustment, Align, Box, DropDown, Entry, SpinButton,
    StringObject, Switch, TextBuffer,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
pub fn change_mpvpaper_wallpaper(
    mpvpaper_changer: WallpaperChangers,
    image: PathBuf,
    monitor: String,
) {
    if let WallpaperChangers::MpvPaper(pause_mode, slideshow, mpv_options) = mpvpaper_changer {
        log::debug!("{}", image.display());
        let mut command = Command::new("mpvpaper");
        command.arg("-o").arg(mpv_options);
        match pause_mode {
            MpvPaperPauseModes::None => {}
            MpvPaperPauseModes::AutoPause => {
                command.arg("--auto-pause");
            }
            MpvPaperPauseModes::AutoStop => {
                command.arg("--auto-stop");
            }
        }
        if slideshow.enable {
            command.arg("-n").arg(slideshow.seconds.to_string());
        }
        command
            .arg(monitor)
            .arg(image)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_mpvpaper_changer_bar(changer_specific_options_box: &Box, settings: Settings) {
    let pause_options_dropdown = DropDown::from_strings(&[
        &gettext("none"),
        &gettext("auto-pause"),
        &gettext("auto-stop"),
    ]);
    pause_options_dropdown.set_margin_top(12);
    pause_options_dropdown.set_margin_start(12);
    pause_options_dropdown.set_margin_bottom(12);
    pause_options_dropdown.set_margin_end(12);
    pause_options_dropdown.set_halign(Align::Start);
    pause_options_dropdown.set_valign(Align::Center);
    settings
        .bind("mpvpaper-pause-option", &pause_options_dropdown, "selected")
        .build();
    changer_specific_options_box.append(&pause_options_dropdown);
    let slideshow_enable_switch = Switch::builder()
        .tooltip_text(gettext("Enable slideshow for the current folder."))
        .has_tooltip(true)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    let adjustment = Adjustment::new(5.0, 1.0, f64::MAX, 1.0, 0.0, 0.0);
    let spin_button = SpinButton::builder()
        .adjustment(&adjustment)
        .numeric(true)
        .has_tooltip(true)
        .tooltip_text(gettext("Slideshow change interval"))
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    changer_specific_options_box.append(&slideshow_enable_switch);
    changer_specific_options_box.append(&spin_button);
    settings
        .bind(
            "mpvpaper-slideshow-enable",
            &slideshow_enable_switch,
            "active",
        )
        .build();
    settings
        .bind("mpvpaper-slideshow-interval", &spin_button, "value")
        .build();

    let mpv_options = create_mpv_options_textbox(&settings);
    changer_specific_options_box.append(&mpv_options);

    slideshow_enable_switch.connect_state_set(clone!(move |_, state| {
        if state {
            let pause_mode = pause_options_dropdown
                .selected_item()
                .and_downcast::<StringObject>()
                .unwrap()
                .string()
                .to_string()
                .parse::<MpvPaperPauseModes>()
                .unwrap();
            let interval = spin_button.value() as u32;
            let options = mpv_options.text().to_string();
            let slideshow_settings = MpvPaperSlideshowSettings {
                enable: state,
                seconds: interval,
            };
            let varient = WallpaperChangers::MpvPaper(pause_mode, slideshow_settings, options);
            let path = settings.string("wallpaper-folder").to_string();
            let monitor = settings.string("selected-monitor-item").to_string();
            log::debug!(
                "{}: {:#?} {} {}",
                gettext("Entered switch callback"),
                varient,
                path,
                monitor
            );
            varient.change(Path::new(&path).to_path_buf(), monitor);
        }
        false.into()
    }));
}

fn create_mpv_options_textbox(settings: &Settings) -> Entry {
    let mpv_options = Entry::builder()
        .placeholder_text(gettext("Additional mpv options"))
        .has_tooltip(true)
        .tooltip_text(gettext(
            "Additional command line options to be sent to mpv.",
        ))
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    let mpv_options_text_buffer = TextBuffer::builder().build();
    settings
        .bind(
            "mpvpaper-additional-options",
            &mpv_options_text_buffer,
            "text",
        )
        .build();

    mpv_options.connect_changed(clone!(
        #[strong]
        mpv_options_text_buffer,
        move |e| {
            let text = &e.text().to_string()[..];
            mpv_options_text_buffer.set_text(text);
        }
    ));
    mpv_options.set_text(
        mpv_options_text_buffer
            .text(
                &mpv_options_text_buffer.start_iter(),
                &mpv_options_text_buffer.end_iter(),
                false,
            )
            .as_str(),
    );
    mpv_options
}
