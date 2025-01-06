use crate::{
    common::{CacheImageFile, GtkPictureFile, RGB, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH},
    database::DatabaseConnection,
    wallpaper_changers::{
        MpvPaperPauseModes, MpvPaperSlideshowSettings, SwaybgModes, WallpaperChanger,
        WallpaperChangers,
    },
};
use async_channel::Sender;
use gtk::{
    self,
    gdk::RGBA,
    gio::{spawn_blocking, ListStore, Settings},
    glib::{self, clone, BoxedAnyObject, Object},
    prelude::*,
    Adjustment, Align, Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, GridView,
    ListItem, ListScrollFlags, SpinButton, StringObject, Switch, TextBuffer,
};
use log::debug;
use std::{
    cell::Ref,
    cmp::Ordering,
    path::{Path, PathBuf},
};
use strum::{IntoEnumIterator, VariantArray};
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
            .expect("The channel must be open");
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
                .expect("The channel must be open");
            if let Ok(i) = DatabaseConnection::check_cache(&file) {
                sender_cache_images
                    .send_blocking(i)
                    .expect("The channel must be open")
            }
        }
        sender_changer_options
            .send_blocking(true)
            .expect("The channel must be open");
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
                "stretch",
                "fit",
                "fill",
                "center",
                "tile",
                "solid_color",
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
                    debug!("Serialized RGB: {}", serialize_struct);
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
            let pause_options_dropdown =
                DropDown::from_strings(&["none", "auto-pause", "auto-stop"]);
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
                .tooltip_text("Enable slideshow for the current folder.")
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
                .tooltip_text("Slideshow change interval")
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
                .tooltip_text("Additional command line options to be sent to mpv.")
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
                log::debug!("Options: {}", text);
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
                        "Entered switch callback: {:#?} {} {}",
                        varient,
                        path,
                        monitor
                    );
                    varient.change(Path::new(&path).to_path_buf(), monitor);
                }
                false.into()
            }));
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
                "Selected changer: {} {} {} {}",
                changer,
                slideshow_enable,
                slideshow_interval,
                options
            );
            changer
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
