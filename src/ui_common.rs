use crate::{
    common::{CacheImageFile, GtkPictureFile, RGB},
    database::DatabaseConnection,
    fs::get_image_files,
    mpvpaper::generate_mpvpaper_changer_bar,
    swaybg::generate_swaybg_changer_bar,
    swww::generate_swww_changer_bar,
    wallpaper_changers::{
        MpvPaperPauseModes, MpvPaperSlideshowSettings, SWWWResizeMode, SWWWScallingFilter,
        SWWWTransitionBezier, SWWWTransitionPosition, SWWWTransitionType, SWWWTransitionWave,
        SwaybgModes, U32Enum, WallpaperChanger, WallpaperChangers,
    },
};
use async_channel::Sender;
use gettextrs::gettext;
use gtk::{
    self,
    gdk::Display,
    gio::{spawn_blocking, ListStore, Settings},
    glib::{BoxedAnyObject, Object},
    prelude::*,
    Box, DropDown, GridView, ListItem, ListScrollFlags, StringObject, Switch,
};
use std::{
    cell::Ref,
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
};

pub const SORT_DROPDOWN_STRINGS: [&str; 2] = ["Date", "Name"];

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
        let files = get_image_files(&path, &sort_dropdown, invert_sort_switch_state);

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

pub fn change_image_button_handlers(
    image_list_store: &ListStore,
    wallpaper_changers_dropdown: &DropDown,
    selected_monitor_dropdown: &DropDown,
    settings: &Settings,
) {
    image_list_store
        .into_iter()
        .filter_map(std::result::Result::ok)
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
            let selected_changer = get_selected_changer(wallpaper_changers_dropdown, settings);
            selected_changer.change(PathBuf::from(&image.path), selected_monitor);
        });
}

pub fn generate_changer_bar(
    changer_specific_options_box: &Box,
    selected_changer: &WallpaperChangers,
    settings: Settings,
) {
    while changer_specific_options_box.first_child().is_some() {
        changer_specific_options_box.remove(&changer_specific_options_box.first_child().unwrap());
    }
    match selected_changer {
        WallpaperChangers::Hyprpaper => {}
        WallpaperChangers::Swaybg(_, _) => {
            generate_swaybg_changer_bar(changer_specific_options_box, &settings);
        }
        WallpaperChangers::MpvPaper(_, _, _) => {
            generate_mpvpaper_changer_bar(changer_specific_options_box, settings);
        }
        WallpaperChangers::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {
            generate_swww_changer_bar(changer_specific_options_box, settings);
        }
    }
}

#[must_use]
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
    image_list_store: &ListStore,
    current_changer: &WallpaperChangers,
    removed_images_list_store: &ListStore,
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
) {
    removed_images_list_store
        .into_iter()
        .filter_map(std::result::Result::ok)
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
        .filter_map(std::result::Result::ok)
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

#[must_use]
pub fn gschema_string_to_string(s: &str) -> String {
    s.replace("\\\"", "\"")
        .replace("\\{", "{")
        .replace("\\}", "}")
}

#[must_use]
pub fn string_to_gschema_string(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace('{', "\\{")
        .replace('}', "\\}")
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

pub fn get_available_monitors() -> Vec<String> {
    let monitors = Display::default().unwrap().monitors();
    monitors
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter_map(|o| o.downcast::<gtk::gdk::Monitor>().ok())
        .filter_map(|m| m.connector())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
}
