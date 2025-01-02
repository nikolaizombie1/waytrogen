use crate::{
    common::{CacheImageFile, GtkPictureFile, RGB},
    database::DatabaseConnection,
    wallpaper_changers::{SwaybgModes, WallpaperChanger, WallpaperChangers},
};
use async_channel::Sender;
use gtk::{
    self,
    gdk::RGBA,
    gio::{spawn_blocking, ListStore, Settings},
    glib::{self, clone, BoxedAnyObject, Object},
    prelude::*,
    Align, Box, Button, ColorDialog, ColorDialogButton, DropDown, GridView, ListItem,
    ListScrollFlags, StringObject, Switch, TextBuffer,
};
use log::debug;
use std::{cell::Ref, path::Path, path::PathBuf};
use strum::IntoEnumIterator;
use which::which;

pub fn generate_image_files(
    path: String,
    sender_cache_images: Sender<CacheImageFile>,
    sort_dropdown: String,
    invert_sort_switch_state: bool,
    sender_changer_options: Sender<bool>,
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

        for file in files {
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
        _ => WallpaperChangers::Hyprpaper,
    }
}

pub fn sort_images(
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    image_list_store: &ListStore,
    image_grid: &GridView,
) {
    match &sort_dropdown
        .selected_item()
        .unwrap()
        .downcast::<StringObject>()
        .unwrap()
        .string()
        .to_string()[..]
    {
        "Name" => {
            image_list_store.sort(|img1, img2| {
                let image1 = img1.downcast_ref::<BoxedAnyObject>().unwrap();
                let image1: Ref<GtkPictureFile> = image1.borrow();
                let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
                let image2: Ref<GtkPictureFile> = image2.borrow();
                if invert_sort_switch.state() {
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
            });
        }
        "Date" => {
            image_list_store.sort(|img1, img2| {
                let image1 = img1.downcast_ref::<BoxedAnyObject>().unwrap();
                let image1: Ref<GtkPictureFile> = image1.borrow();
                let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
                let image2: Ref<GtkPictureFile> = image2.borrow();
                if invert_sort_switch.state() {
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
            });
        }
        _ => {}
    }
    image_grid.scroll_to(0, ListScrollFlags::FOCUS, None);
}

pub fn hide_unsupported_files(image_list_store: ListStore, current_changer: WallpaperChangers) {
    let images = image_list_store
        .into_iter()
        .filter_map(|o| o.ok())
        .collect::<Vec<_>>();
    debug!("Num of objects in list store: {}", images.len());
    let images = images
        .into_iter()
        .filter_map(|o| o.downcast::<BoxedAnyObject>().ok())
        .collect::<Vec<_>>();
    debug!("Num of ListItems in list store: {}", images.len());
    images.into_iter().for_each(|b| {
        let image_file: Ref<GtkPictureFile> = b.borrow();
        let button = image_file
            .picture
            .parent()
            .and_upcast::<Object>()
            .and_downcast::<Button>()
            .unwrap();
        if current_changer.accepted_formats().into_iter().any(|f| {
            f == Path::new(&image_file.chache_image_file.path)
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        }) {
            button.set_sensitive(true);
        } else {
            button.set_sensitive(false);
        }
    });
}
