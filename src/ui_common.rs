use crate::{
    common::{CacheImageFile, GtkPictureFile, RGB},
    database::DatabaseConnection,
    wallpaper_changers::{
        MpvPaperPauseModes, MpvPaperSlideshowSettings, SWWWResizeMode, SWWWScallingFilter,
        SWWWTransitionBezier, SWWWTransitionPosition, SWWWTransitionType, SWWWTransitionWave,
        SwaybgModes, U32Enum, WallpaperChanger, WallpaperChangers,
    },
};
use async_channel::Sender;
use gettextrs::*;
use gtk::{
    self,
    gdk::RGBA,
    gio::{spawn_blocking, ListStore, Settings},
    glib::{self, clone, BoxedAnyObject, Object},
    prelude::*,
    Adjustment, Align, Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, GridView,
    Label, ListItem, ListScrollFlags, SpinButton, StringObject, Switch, TextBuffer, Window,
};
use log::debug;
use std::{
    cell::Ref,
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
};
use strum::IntoEnumIterator;
use which::which;

pub fn generate_image_files(
    path: String,
    sender_cache_images: Sender<CacheImageFile>,
    sort_dropdown: String,
    invert_sort_switch_state: bool,
    sender_changer_options: Sender<bool>,
    sender_images_loading_progress_bar: Sender<f64>,
) {
    spawn_blocking(move || {
        sender_changer_options
            .send_blocking(false)
            .unwrap_or_else(|_| panic!("{}", gettext("The channel must be open")));
        let mut files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .map(|d| d.path().to_path_buf())
            .filter(|p| {
                WallpaperChangers::all_accepted_formats().iter().any(|f| {
                    f == p
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                })
            })
            .collect::<Vec<_>>();
        match &sort_dropdown.to_lowercase()[..] {
            "name" => {
                files.sort_by(|f1, f2| {
                    if invert_sort_switch_state {
                        f1.file_name().partial_cmp(&f2.file_name()).unwrap()
                    } else {
                        f2.file_name().partial_cmp(&f1.file_name()).unwrap()
                    }
                });
            }
            "date" => {
                files.sort_by(|f1, f2| {
                    if invert_sort_switch_state {
                        f1.metadata()
                            .unwrap()
                            .created()
                            .unwrap()
                            .partial_cmp(&f2.metadata().unwrap().created().unwrap())
                            .unwrap()
                    } else {
                        f2.metadata()
                            .unwrap()
                            .created()
                            .unwrap()
                            .partial_cmp(&f1.metadata().unwrap().created().unwrap())
                            .unwrap()
                    }
                });
            }
            _ => {}
        }

        for (index, file) in files.iter().enumerate() {
            sender_images_loading_progress_bar
                .send_blocking((index as f64) / (files.len() as f64))
                .unwrap_or_else(|_| panic!("{}", gettext("The channel must be open")));
            if let Ok(i) = DatabaseConnection::check_cache(file) {
                sender_cache_images
                    .send_blocking(i)
                    .unwrap_or_else(|_| panic!("{}", gettext("The channel must be open")));
            }
        }
        sender_changer_options
            .send_blocking(true)
            .unwrap_or_else(|_| panic!("{}", gettext("The channel must be open")));
    });
}

pub fn get_available_wallpaper_changers() -> Vec<WallpaperChangers> {
    let mut available_changers = vec![];
    for changer in WallpaperChangers::iter() {
        if which(changer.to_string().to_lowercase()).is_ok() {
            available_changers.push(changer);
        }
    }
    available_changers
}

pub fn change_image_button_handlers(
    image_list_store: ListStore,
    wallpaper_changers_dropdown: DropDown,
    selected_monitor_dropdown: DropDown,
    settings: &Settings,
) {
    image_list_store
        .into_iter()
        .filter_map(|o| o.ok())
        .filter_map(|o| o.downcast::<ListItem>().ok())
        .for_each(|li| {
            let entry = li.item().and_downcast::<BoxedAnyObject>().unwrap();
            let image: Ref<CacheImageFile> = entry.borrow();
            let selected_monitor = selected_monitor_dropdown
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string()
                .to_string();
            let selected_changer = get_selected_changer(&wallpaper_changers_dropdown, settings);
            selected_changer.change(PathBuf::from(&image.path), selected_monitor);
        });
}

pub fn generate_changer_bar(
    changer_specific_options_box: Box,
    selected_changer: WallpaperChangers,
    settings: Settings,
) {
    while changer_specific_options_box.first_child().is_some() {
        changer_specific_options_box.remove(&changer_specific_options_box.first_child().unwrap());
    }
    match selected_changer {
        WallpaperChangers::Hyprpaper => {}
        WallpaperChangers::Swaybg(_, _) => {
            let dropdown = DropDown::from_strings(&[
                &gettext("stretch"),
                &gettext("fit"),
                &gettext("fill"),
                &gettext("center"),
                &gettext("tile"),
                &gettext("solid_color"),
            ]);
            dropdown.set_halign(Align::Start);
            dropdown.set_valign(Align::Center);
            dropdown.set_margin_top(12);
            dropdown.set_margin_start(12);
            dropdown.set_margin_bottom(12);
            dropdown.set_margin_end(12);
            changer_specific_options_box.append(&dropdown);
            let color_dialog = ColorDialog::builder().with_alpha(false).build();
            let color_picker = ColorDialogButton::builder()
                .halign(Align::Start)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .dialog(&color_dialog)
                .build();
            changer_specific_options_box.append(&color_picker);
            settings.bind("swaybg-mode", &dropdown, "selected").build();
            let rgb_text_buffer = TextBuffer::builder().build();
            color_picker.connect_rgba_notify(clone!(
                #[weak]
                settings,
                move |b| {
                    let rgba = b.rgba();
                    let serialize_struct = RGB {
                        red: rgba.red(),
                        green: rgba.green(),
                        blue: rgba.blue(),
                    }
                    .to_string();
                    debug!("{}: {}", gettext("Serialized RGB"), serialize_struct);
                    rgb_text_buffer.set_text(&serialize_struct);
                    settings
                        .bind("swaybg-color", &rgb_text_buffer, "text")
                        .build();
                }
            ));
            let rgb = settings
                .string("swaybg-color")
                .to_string()
                .parse::<RGB>()
                .unwrap();
            color_picker.set_rgba(
                &RGBA::builder()
                    .red(rgb.red)
                    .green(rgb.green)
                    .blue(rgb.blue)
                    .build(),
            );
        }
        WallpaperChangers::MpvPaper(_, _, _) => {
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
            let mpv_options = Entry::builder()
                .placeholder_text("Additional mpv options")
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
            changer_specific_options_box.append(&mpv_options);
            let mpv_options_text_buffer_copy = mpv_options_text_buffer.clone();
            mpv_options.connect_changed(clone!(move |e| {
                let text = &e.text().to_string()[..];
                mpv_options_text_buffer_copy.set_text(text);
            }));
            mpv_options.set_text(
                mpv_options_text_buffer
                    .text(
                        &mpv_options_text_buffer.start_iter(),
                        &mpv_options_text_buffer.end_iter(),
                        false,
                    )
                    .as_str(),
            );
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
                    let varient =
                        WallpaperChangers::MpvPaper(pause_mode, slideshow_settings, options);
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
        WallpaperChangers::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {
            let resize_dropdown =
                DropDown::from_strings(&[&gettext("no"), &gettext("crop"), &gettext("fit")]);
            resize_dropdown.set_margin_top(12);
            resize_dropdown.set_margin_start(12);
            resize_dropdown.set_margin_bottom(12);
            resize_dropdown.set_margin_end(12);
            resize_dropdown.set_halign(Align::Start);
            resize_dropdown.set_valign(Align::Center);
            changer_specific_options_box.append(&resize_dropdown);
            settings
                .bind("swww-resize", &resize_dropdown, "selected")
                .build();
            let color_dialog = ColorDialog::builder().with_alpha(false).build();
            let color_picker = ColorDialogButton::builder()
                .halign(Align::Start)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .dialog(&color_dialog)
                .build();
            let rgb_text_buffer = TextBuffer::builder().build();
            color_picker.connect_rgba_notify(clone!(
                #[weak]
                settings,
                move |b| {
                    let rgba = b.rgba();
                    let serialize_struct = RGB {
                        red: rgba.red(),
                        green: rgba.green(),
                        blue: rgba.blue(),
                    }
                    .to_string();
                    rgb_text_buffer.set_text(&serialize_struct);
                    settings
                        .bind("swww-fill-color", &rgb_text_buffer, "text")
                        .build();
                }
            ));
            let rgb = settings
                .string("swww-fill-color")
                .to_string()
                .parse::<RGB>()
                .unwrap();
            color_picker.set_rgba(
                &RGBA::builder()
                    .red(rgb.red)
                    .green(rgb.green)
                    .blue(rgb.blue)
                    .build(),
            );
            changer_specific_options_box.append(&color_picker);
            let advanced_settings_window = Window::builder()
                .title(gettext("SWWW Advanced Image Settings"))
                .hide_on_close(true)
                .build();
            let advanced_settings_button = Button::builder()
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .label(gettext("Advanced Settings"))
                .halign(Align::Start)
                .valign(Align::Center)
                .build();
            changer_specific_options_box.append(&advanced_settings_button);
            advanced_settings_button.connect_clicked(
                move |_| {
            let advanced_settings_window_box = Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .hexpand(true)
                .vexpand(true)
                .build();
            advanced_settings_window.present();
            advanced_settings_window.set_child(Some(&advanced_settings_window_box));
            let filter_options_label = Label::builder()
                .label(gettext("Scalling filter"))
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            let filter_dropdown = DropDown::from_strings(&[
                &gettext("nearest"),
                &gettext("bilinear"),
                &gettext("catmullrom"),
                &gettext("mitchell"),
                &gettext("lanczos3"),
            ]);
            filter_dropdown.set_margin_top(12);
            filter_dropdown.set_margin_start(12);
            filter_dropdown.set_margin_bottom(12);
            filter_dropdown.set_margin_end(12);
            filter_dropdown.set_halign(Align::Start);
            filter_dropdown.set_valign(Align::Center);
            settings
                .bind("swww-scaling-filter", &filter_dropdown, "selected")
                .build();
            let transition_type_label = Label::builder()
                .label(gettext("Transition type"))
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            let transition_type_dropdown = DropDown::from_strings(&[
                &gettext("none"), &gettext("simple"), &gettext("fade"), &gettext("left"), &gettext("right"), &gettext("top"), &gettext("bottom"), &gettext("wipe"), &gettext("wave"), &gettext("grow"),
                &gettext("center"), &gettext("any"), &gettext("outer"), &gettext("random"),
            ]);
            transition_type_dropdown.set_margin_top(12);
            transition_type_dropdown.set_margin_start(12);
            transition_type_dropdown.set_margin_bottom(12);
            transition_type_dropdown.set_margin_end(12);
            transition_type_dropdown.set_halign(Align::Start);
            transition_type_dropdown.set_valign(Align::Center);
            settings
                .bind(
                    "swww-transition-type",
                    &transition_type_dropdown,
                    "selected",
                )
                .build();

            let filter_and_transition_box = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .hexpand(true)
                .vexpand(true)
                .build();

            filter_and_transition_box.append(&filter_options_label);
            filter_and_transition_box.append(&filter_dropdown);
            filter_and_transition_box.append(&transition_type_label);
            filter_and_transition_box.append(&transition_type_dropdown);
            advanced_settings_window_box.append(&filter_and_transition_box);

            let transition_step_label = Label::builder()
                .label(gettext("Transition step"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let transition_step_adjustment =
                Adjustment::new(90.0, 0.0, u8::MAX as f64, 1.0, 0.0, 0.0);
            let transition_step_spinbutton = SpinButton::builder()
                .adjustment(&transition_step_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();

            settings
                .bind("swww-transition-step", &transition_step_spinbutton, "value")
                .build();

            let transition_duration_label = Label::builder()
                .label(gettext("Transition duration"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();
            let transition_duration_adjustment =
                Adjustment::new(3.0, 0.0, u32::MAX as f64, 1.0, 0.0, 0.0);
            let transition_duration_spinbutton = SpinButton::builder()
                .adjustment(&transition_duration_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings
                .bind(
                    "swww-transition-duration",
                    &transition_duration_spinbutton,
                    "value",
                )
                .build();




            let transition_angle_label = Label::builder()
                .label(gettext("Transition angle"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();
            let transition_angle_adjustment = Adjustment::new(45.0, 0.0, 270.0, 1.0, 0.0, 0.0);
            let transition_angle_spinbutton = SpinButton::builder()
                .adjustment(&transition_angle_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings
                .bind(
                    "swww-transition-angle",
                    &transition_angle_spinbutton,
                    "value",
                )
                .build();

            let transition_step_duration_angle_box = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .hexpand(true)
                .vexpand(true)
                .build();

            transition_step_duration_angle_box.append(&transition_step_label);
            transition_step_duration_angle_box.append(&transition_step_spinbutton);
            transition_step_duration_angle_box.append(&transition_duration_label);
            transition_step_duration_angle_box.append(&transition_duration_spinbutton);
            transition_step_duration_angle_box.append(&transition_angle_label);
            transition_step_duration_angle_box.append(&transition_angle_spinbutton);
            advanced_settings_window_box.append(&transition_step_duration_angle_box);

            let transition_position_label = Label::builder()
                .label(gettext("Transition position"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let transition_position_entry = Entry::builder()
                .placeholder_text(gettext("Transition position"))
                .has_tooltip(true)
                .tooltip_text(gettext("Can either be floating point number between 0 and 0.99, integer coordinate like 200,200 or one of the following: center, top, left, right, bottom, top-left, top-right, bottom-left or bottom-right."))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Start)
                .valign(Align::Center)
                .build();

            let transition_position_entry_text_buffer = TextBuffer::builder().build();
            settings
                .bind(
                    "swww-transition-position",
                    &transition_position_entry_text_buffer,
                    "text",
                )
                .build();

            transition_position_entry.set_text(
                transition_position_entry_text_buffer.text(&transition_position_entry_text_buffer.start_iter(), &transition_position_entry_text_buffer.end_iter(), false).as_ref()
            );

            transition_position_entry.connect_changed(move |e| {
                let text = e.text().to_string();
                if SWWWTransitionPosition::new(&text).is_ok() { transition_position_entry_text_buffer.set_text(&text) }
            });

            let invert_y_label = Label::builder()
                .label(gettext("Invert Y"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let invert_y_switch = Switch::builder()
                .tooltip_text(gettext("Invert y position in transition position flag"))
                .has_tooltip(true)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Start)
                .valign(Align::Center)
                .build();

            settings.bind("swww-invert-y", &invert_y_switch, "active").build();

            let transition_wave_label = Label::builder()
                .label(gettext("Transition wave"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let transition_wave_width_adjustment =
                Adjustment::new(20.0, 0.0, u32::MAX as f64, 1.0, 0.0, 0.0);
            let transition_wave_width_spinbutton = SpinButton::builder()
                .adjustment(&transition_wave_width_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();

            settings.bind("swww-transition-wave-width", &transition_wave_width_spinbutton, "value").build();

            let transition_wave_height_adjustment =
                Adjustment::new(20.0, 0.0, u32::MAX as f64, 1.0, 0.0, 0.0);
            let transition_wave_height_spinbutton = SpinButton::builder()
                .adjustment(&transition_wave_height_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();

            settings.bind("swww-transition-wave-height", &transition_wave_height_spinbutton, "value").build();

            let transition_position_invert_y_wave_box = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .hexpand(true)
                .vexpand(true)
                .build();

            transition_position_invert_y_wave_box.append(&transition_position_label);
            transition_position_invert_y_wave_box.append(&transition_position_entry);
            transition_position_invert_y_wave_box.append(&invert_y_label);
            transition_position_invert_y_wave_box.append(&invert_y_switch);
            transition_position_invert_y_wave_box.append(&transition_wave_label);
            transition_position_invert_y_wave_box.append(&transition_wave_width_spinbutton);
            transition_position_invert_y_wave_box.append(&transition_wave_height_spinbutton);
            advanced_settings_window_box.append(&transition_position_invert_y_wave_box);

            let transition_bezier_label = Label::builder()
                .label(gettext("Transition bezier"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let transition_bezier_adjustments = Adjustment::new(0.0, f64::MIN, f64::MAX, 0.01, 0.0, 0.0);
            let transition_bezier_p0_spinbutton = SpinButton::builder()
                .adjustment(&transition_bezier_adjustments)
                .numeric(true)
                .digits(2)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings.bind("swww-transition-bezier-p0", &transition_bezier_p0_spinbutton, "value").build();
            let transition_bezier_p1_spinbutton = SpinButton::builder()
                .adjustment(&transition_bezier_adjustments)
                .digits(2)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings.bind("swww-transition-bezier-p1", &transition_bezier_p1_spinbutton, "value").build();
            let transition_bezier_p2_spinbutton = SpinButton::builder()
                .adjustment(&transition_bezier_adjustments)
                .numeric(true)
                .digits(2)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings.bind("swww-transition-bezier-p2", &transition_bezier_p2_spinbutton, "value").build();
            let transition_bezier_p3_spinbutton = SpinButton::builder()
                .adjustment(&transition_bezier_adjustments)
                .numeric(true)
                .digits(2)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();
            settings.bind("swww-transition-bezier-p3", &transition_bezier_p3_spinbutton, "value").build();

            let transition_bezier_fps_box = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .hexpand(true)
                .vexpand(true)
                .build();

            let transition_frames_per_second_label = Label::builder()
                .label(gettext("Transition FPS"))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Center)
                .valign(Align::Center)
                .build();

            let transition_frames_per_second_adjustment =
                Adjustment::new(30.0, 1.0, u32::MAX as f64, 1.0, 0.0, 0.0);

            let transition_frames_per_second_spinbutton = SpinButton::builder()
                .adjustment(&transition_frames_per_second_adjustment)
                .numeric(true)
                .halign(Align::Center)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .build();

            settings.bind("swww-transition-fps", &transition_frames_per_second_spinbutton, "value").build();

            transition_bezier_fps_box.append(&transition_bezier_label);
            transition_bezier_fps_box.append(&transition_bezier_p0_spinbutton);
            transition_bezier_fps_box.append(&transition_bezier_p1_spinbutton);
            transition_bezier_fps_box.append(&transition_bezier_p2_spinbutton);
            transition_bezier_fps_box.append(&transition_bezier_p3_spinbutton);
            transition_bezier_fps_box.append(&transition_frames_per_second_label);
            transition_bezier_fps_box.append(&transition_frames_per_second_spinbutton);
            advanced_settings_window_box.append(&transition_bezier_fps_box);

		    let window_hide_button = Button::builder().
                margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .label(gettext("Confirm"))
                .halign(Align::End)
                .valign(Align::Center)
                .build();

            let restore_defaults_button = Button::builder().
                margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .label(gettext("Restore Defaults"))
                .halign(Align::End)
                .valign(Align::Center)
                .build();


		    let window_control_box = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
			    .halign(Align::End)
                .valign(Align::Center)
                .hexpand(true)
                .vexpand(true)
                .build();


            restore_defaults_button.connect_clicked(move |_| {
                filter_dropdown.set_selected(SWWWScallingFilter::default().to_u32());
                transition_step_spinbutton.set_value(90.0);
                transition_duration_spinbutton.set_value(3.0);
                transition_angle_spinbutton.set_value(45.0);
                transition_position_entry.set_text(&SWWWTransitionPosition::default().to_string());
                invert_y_switch.set_state(false);
                transition_wave_width_spinbutton.set_value(200.0);
                transition_wave_height_spinbutton.set_value(200.0);
                transition_bezier_p0_spinbutton.set_value(SWWWTransitionBezier::default().p0);
                transition_bezier_p1_spinbutton.set_value(SWWWTransitionBezier::default().p1);
                transition_bezier_p2_spinbutton.set_value(SWWWTransitionBezier::default().p2);
                transition_bezier_p3_spinbutton.set_value(SWWWTransitionBezier::default().p3);
                transition_frames_per_second_spinbutton.set_value(30.0);
            });

		    window_hide_button.connect_clicked(clone!(
			#[weak]
			advanced_settings_window,
			move |_| {
			advanced_settings_window.set_visible(false);
		    }));
            window_control_box.append(&restore_defaults_button);
		    window_control_box.append(&window_hide_button);
		    advanced_settings_window_box.append(&window_control_box);
                }
            );
        }
    }
}

pub fn get_selected_changer(
    wallpaper_changers_dropdown: &DropDown,
    settings: &Settings,
) -> WallpaperChangers {
    let selected_item = wallpaper_changers_dropdown
        .selected_item()
        .unwrap()
        .downcast::<StringObject>()
        .unwrap()
        .string()
        .to_string()
        .to_lowercase();
    match &selected_item[..] {
        "hyprpaper" => WallpaperChangers::Hyprpaper,
        "swaybg" => {
            let mode = SwaybgModes::from_u32(settings.uint("swaybg-mode"));
            let rgb = settings.string("swaybg-color").to_string();
            WallpaperChangers::Swaybg(mode, rgb)
        }
        "mpvpaper" => {
            let pause_mode = MpvPaperPauseModes::from_u32(settings.uint("mpvpaper-pause-option"));
            let slideshow_enable = settings.boolean("mpvpaper-slideshow-enable");
            let slideshow_interval = settings.double("mpvpaper-slideshow-interval") as u32;
            let options = settings.string("mpvpaper-additional-options").to_string();
            let changer = WallpaperChangers::MpvPaper(
                pause_mode,
                MpvPaperSlideshowSettings {
                    enable: slideshow_enable,
                    seconds: slideshow_interval,
                },
                options.clone(),
            );
            log::debug!(
                "{}: {} {} {} {}",
                gettext("Selected changer"),
                changer,
                slideshow_enable,
                slideshow_interval,
                options
            );
            changer
        }
        "swww" => {
            let resize = SWWWResizeMode::from_u32(settings.uint("swww-resize"));
            let fill_color = RGB::from_str(settings.string("swww-fill-color").as_str()).unwrap();
            let scaling_filter = SWWWScallingFilter::from_u32(settings.uint("swww-scaling-filter"));
            let transition_type =
                SWWWTransitionType::from_u32(settings.uint("swww-transition-type"));
            let transition_step = settings.double("swww-transition-step") as u8;
            let transition_duration = settings.double("swww-transition-duration") as u32;
            let transition_angle = settings.double("swww-transition-angle") as u16;
            let transition_position =
                SWWWTransitionPosition::new(settings.string("swww-transition-position").as_str())
                    .unwrap();
            let invert_y = settings.boolean("swww-invert-y");
            let transition_wave = SWWWTransitionWave {
                width: settings.double("swww-transition-wave-width") as u32,
                height: settings.double("swww-transition-wave-height") as u32,
            };
            let transition_bezier = SWWWTransitionBezier {
                p0: settings.double("swww-transition-bezier-p0"),
                p1: settings.double("swww-transition-bezier-p1"),
                p2: settings.double("swww-transition-bezier-p2"),
                p3: settings.double("swww-transition-bezier-p3"),
            };
            let transition_fps = settings.uint("swww-transition-fps");
            WallpaperChangers::Swww(
                resize,
                fill_color,
                scaling_filter,
                transition_type,
                transition_step,
                transition_duration,
                transition_fps,
                transition_angle,
                transition_position,
                invert_y,
                transition_bezier,
                transition_wave,
            )
        }
        _ => WallpaperChangers::Hyprpaper,
    }
}

pub fn sort_images(
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    image_list_store: &ListStore,
    image_grid: &GridView,
) {
    image_list_store.sort(compare_image_list_items_by_sort_selection_comparitor(
        sort_dropdown.clone(),
        invert_sort_switch.clone(),
    ));
    image_grid.scroll_to(0, ListScrollFlags::FOCUS, None);
}

pub fn hide_unsupported_files(
    image_list_store: ListStore,
    current_changer: WallpaperChangers,
    removed_images_list_store: &ListStore,
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
) {
    removed_images_list_store
        .into_iter()
        .filter_map(|o| o.ok())
        .for_each(|o| {
            let b = o.downcast::<BoxedAnyObject>().unwrap();
            image_list_store.insert_sorted(
                &b,
                compare_image_list_items_by_sort_selection_comparitor(
                    sort_dropdown.clone(),
                    invert_sort_switch.clone(),
                ),
            );
        });
    removed_images_list_store.remove_all();
    image_list_store
        .into_iter()
        .filter_map(|o| o.ok())
        .for_each(|o| {
            let item = o.clone().downcast::<BoxedAnyObject>().unwrap();
            let image_file: Ref<GtkPictureFile> = item.borrow();
            if !current_changer.accepted_formats().into_iter().any(|f| {
                f == Path::new(&image_file.chache_image_file.path)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            }) {
                removed_images_list_store.append(&item);
                image_list_store.remove(image_list_store.find(&o).unwrap());
            }
        });
}

pub fn gschema_string_to_string(s: &str) -> String {
    s.replace("\\\"", "\"")
        .replace("\\{", "{")
        .replace("\\}", "}")
}

pub fn string_to_gschema_string(s: &str) -> String {
    s.replace("\"", "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
}

pub fn compare_image_list_items_by_sort_selection_comparitor(
    sort_dropdown: DropDown,
    invert_sort_switch: Switch,
) -> impl Fn(&Object, &Object) -> Ordering {
    move |img1, img2| {
        let invert_sort_switch_state = invert_sort_switch.state();
        match &sort_dropdown
            .selected_item()
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .string()
            .to_lowercase()
            .to_string()[..]
        {
            "name" => {
                compare_image_list_items_by_name_comparitor(invert_sort_switch_state)(img1, img2)
            }
            _ => compare_image_list_items_by_date_comparitor(invert_sort_switch_state)(img1, img2),
        }
    }
}

pub fn compare_image_list_items_by_name_comparitor(
    invert_sort_switch_state: bool,
) -> impl Fn(&Object, &Object) -> Ordering {
    move |img1, img2| {
        let image1 = img1.downcast_ref::<BoxedAnyObject>().unwrap();
        let image1: Ref<GtkPictureFile> = image1.borrow();
        let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
        let image2: Ref<GtkPictureFile> = image2.borrow();
        if invert_sort_switch_state {
            image1
                .chache_image_file
                .name
                .partial_cmp(&image2.chache_image_file.name)
                .unwrap()
        } else {
            image2
                .chache_image_file
                .name
                .partial_cmp(&image1.chache_image_file.name)
                .unwrap()
        }
    }
}

pub fn compare_image_list_items_by_date_comparitor(
    invert_sort_switch_state: bool,
) -> impl Fn(&Object, &Object) -> Ordering {
    move |img1, img2| {
        let image1 = img1.downcast_ref::<BoxedAnyObject>().unwrap();
        let image1: Ref<GtkPictureFile> = image1.borrow();
        let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
        let image2: Ref<GtkPictureFile> = image2.borrow();
        if invert_sort_switch_state {
            image1
                .chache_image_file
                .date
                .partial_cmp(&image2.chache_image_file.date)
                .unwrap()
        } else {
            image2
                .chache_image_file
                .date
                .partial_cmp(&image1.chache_image_file.date)
                .unwrap()
        }
    }
}
