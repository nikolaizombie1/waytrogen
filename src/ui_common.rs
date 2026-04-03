use crate::{
    changers::{
        gslapper::generate_gslapper_changer_bar, hyprpaper::generate_hyprpaper_changer_bar,
        mpvpaper::generate_mpvpaper_changer_bar, swaybg::generate_swaybg_changer_bar,
        awww::generate_awww_changer_bar,
    },
    common::{CacheImageFile, GtkPictureFile, RGB},
    database::DatabaseConnection,
    fs::get_image_files,
    wallpaper_changers::{
        GSllapperPauseMode, GSllapperScaleMode, HyprpaperFitModes, MpvPaperPauseModes,
        MpvPaperSlideshowSettings, AWWWResizeMode, AWWWScallingFilter, AWWWTransitionBezier,
        AWWWTransitionPosition, AWWWTransitionType, AWWWTransitionWave, SwaybgModes, U32Enum,
        WallpaperChanger, WallpaperChangers,
    },
};
use async_channel::Sender;
use gettextrs::gettext;
use gtk::{
    self,
    gdk::Display,
    gio::{spawn_blocking, ListStore, Settings},
    glib::Object,
    prelude::*,
    Box, DropDown, GridView, ListItem, ListScrollFlags, StringObject, Switch,
};

use chrono::prelude::*;

use log::{debug, trace};
use std::{
    cell::Ref,
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
};

pub const SORT_DROPDOWN_STRINGS: [&str; 2] = ["Date", "Name"];
pub const DEFAULT_MARGIN: i32 = 12;

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

        let time_before_load = Local::now();
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
        let image_load_time_milliseconds = (Local::now() - time_before_load).num_milliseconds();
        trace!(
            "Image grid took {} milliseconds to load {} images",
            image_load_time_milliseconds,
            files.len()
        );
        trace!(
            "Average time per image: {:.2} nanoseconds",
            (image_load_time_milliseconds as f64 / files.len() as f64) * 1000.0
        );
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
            let entry = li.item().and_downcast::<GtkPictureFile>().unwrap();
            let image: Ref<CacheImageFile> = entry.cache_image_file().borrow();
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
        WallpaperChangers::Hyprpaper(_) => {
            generate_hyprpaper_changer_bar(changer_specific_options_box, &settings)
        }
        WallpaperChangers::Swaybg(_, _) => {
            generate_swaybg_changer_bar(changer_specific_options_box, &settings);
        }
        WallpaperChangers::MpvPaper(_, _, _) => {
            generate_mpvpaper_changer_bar(changer_specific_options_box, settings);
        }
        WallpaperChangers::Awww(_, _, _, _, _, _, _, _, _, _, _, _) => {
            generate_awww_changer_bar(changer_specific_options_box, settings);
        }
        WallpaperChangers::GSlapper(_, _, _, _) => {
            generate_gslapper_changer_bar(changer_specific_options_box, settings);
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
            debug!(
                "{}: {} {} {} {}",
                gettext("Selected changer"),
                changer,
                slideshow_enable,
                slideshow_interval,
                options
            );
            changer
        }
        "awww" => {
            let resize = AWWWResizeMode::from_u32(settings.uint("awww-resize"));
            let fill_color = RGB::from_str(settings.string("awww-fill-color").as_str()).unwrap();
            let scaling_filter = AWWWScallingFilter::from_u32(settings.uint("awww-scaling-filter"));
            let transition_type =
                AWWWTransitionType::from_u32(settings.uint("awww-transition-type"));
            let transition_step = settings.double("awww-transition-step") as u8;
            let transition_duration = settings.double("awww-transition-duration") as u32;
            let transition_angle = settings.double("awww-transition-angle") as u16;
            let transition_position =
                AWWWTransitionPosition::new(settings.string("awww-transition-position").as_str())
                    .unwrap();
            let invert_y = settings.boolean("awww-invert-y");
            let transition_wave = AWWWTransitionWave {
                width: settings.double("awww-transition-wave-width") as u32,
                height: settings.double("awww-transition-wave-height") as u32,
            };
            let transition_bezier = AWWWTransitionBezier {
                p0: settings.double("awww-transition-bezier-p0"),
                p1: settings.double("awww-transition-bezier-p1"),
                p2: settings.double("awww-transition-bezier-p2"),
                p3: settings.double("awww-transition-bezier-p3"),
            };
            let transition_fps = settings.uint("awww-transition-fps");
            WallpaperChangers::Awww(
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
        "gslapper" => {
            let scale_mode = GSllapperScaleMode::from_u32(settings.uint("gslapper-scale-mode"));
            let pause_mode = GSllapperPauseMode::from_u32(settings.uint("gslapper-pause-mode"));
            let loop_video = settings.boolean("gslapper-loop");
            let additional_options = settings.string("gslapper-additional-options").to_string();
            let changer = WallpaperChangers::GSlapper(
                scale_mode,
                pause_mode,
                loop_video,
                additional_options.clone(),
            );
            debug!(
                "{}: {} loop={} options={}",
                gettext("Selected changer"),
                changer,
                loop_video,
                additional_options
            );
            changer
        }
        _ => {
            let fit_mode = settings.uint("hyprpaper-fit-mode");
            let fit_mode = HyprpaperFitModes::from_u32(fit_mode);
            WallpaperChangers::Hyprpaper(fit_mode)
        }
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
    if image_list_store.into_iter().len() != 0 {
        image_grid.scroll_to(0, ListScrollFlags::FOCUS, None);
    }
}

pub fn hide_unsupported_files(
    image_list_store: &ListStore,
    current_changer: &WallpaperChangers,
    removed_images_list_store: &ListStore,
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    name_filter: &str,
) {
    removed_images_list_store
        .into_iter()
        .filter_map(std::result::Result::ok)
        .for_each(|o| {
            let b = o.downcast::<GtkPictureFile>().unwrap();
            image_list_store.insert_sorted(
                &b,
                compare_image_list_items_by_sort_selection_comparitor(
                    sort_dropdown.clone(),
                    invert_sort_switch.clone(),
                ),
            );
        });
    removed_images_list_store.remove_all();
    let ls = image_list_store
        .into_iter()
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();
    debug!("Filtered list store size: {}", ls.len());

    for o in ls {
        let image_file = o.clone().downcast::<GtkPictureFile>().unwrap();
        if !current_changer.accepted_formats().contains(
            &Path::new(&image_file.cache_image_file().borrow().path)
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_owned(),
        ) || !&image_file
            .cache_image_file()
            .borrow()
            .name
            .to_lowercase()
            .contains(&name_filter.to_lowercase())
        {
            debug!(
                "Image name: {}, Name Filter: {name_filter}, Contains: {}",
                &image_file.cache_image_file().borrow().name,
                &image_file
                    .cache_image_file()
                    .borrow()
                    .name
                    .contains(name_filter)
            );
            transfer_and_remove_image(removed_images_list_store, image_list_store, &o, &image_file);
        }
    }
}

fn transfer_and_remove_image(
    removed_images_list_store: &ListStore,
    image_list_store: &ListStore,
    o: &Object,
    image_file: &GtkPictureFile,
) {
    removed_images_list_store.append(image_file);
    image_list_store.remove(image_list_store.find(o).unwrap());
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
        let image1 = img1.downcast_ref::<GtkPictureFile>().unwrap();
        let image2 = img2.downcast_ref::<GtkPictureFile>().unwrap();
        if invert_sort_switch_state {
            image1
                .cache_image_file()
                .borrow()
                .name
                .partial_cmp(&image2.cache_image_file().borrow().name)
                .unwrap()
        } else {
            image2
                .cache_image_file()
                .borrow()
                .name
                .partial_cmp(&image1.cache_image_file().borrow().name)
                .unwrap()
        }
    }
}

pub fn compare_image_list_items_by_date_comparitor(
    invert_sort_switch_state: bool,
) -> impl Fn(&Object, &Object) -> Ordering {
    move |img1, img2| {
        let image1 = img1.downcast_ref::<GtkPictureFile>().unwrap();
        let image2 = img2.downcast_ref::<GtkPictureFile>().unwrap();
        if invert_sort_switch_state {
            image1
                .cache_image_file()
                .borrow()
                .date
                .partial_cmp(&image2.cache_image_file().borrow().date)
                .unwrap()
        } else {
            image2
                .cache_image_file()
                .borrow()
                .date
                .partial_cmp(&image1.cache_image_file().borrow().date)
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
