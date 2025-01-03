use std::{cell::Ref, path::PathBuf};

use async_channel::{Receiver, Sender};
use gtk::{
    self,
    gdk::{Display, Texture},
    gio::{spawn_blocking, Cancellable, ListStore, Settings},
    glib::{self, clone, spawn_future_local, BoxedAnyObject, Bytes},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, DropDown, FileDialog, GridView, Label,
    ListItem, Orientation, Picture, ScrolledWindow, SignalListItemFactory, SingleSelection,
    Spinner, StringObject, Switch, Text, TextBuffer,
};
use log::debug;
use waytrogen::{
    common::{CacheImageFile, GtkPictureFile, Wallpaper, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH},
    ui_common::{
        change_image_button_handlers, generate_changer_bar, generate_image_files,
        get_available_wallpaper_changers, get_selected_changer, gschema_string_to_string,
        hide_unsupported_files, sort_images, string_to_gschema_string,
    },
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
};

const APP_ID: &str = "org.Waytrogen.Waytrogen";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    stderrlog::new()
        .module(module_path!())
        .verbosity(5)
        .init()
        .unwrap();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Waytrogen")
        .build();

    window.present();

    let settings = Settings::new(APP_ID);

    let image_list_store = ListStore::new::<BoxedAnyObject>();

    let selection = SingleSelection::new(Some(image_list_store.clone()));
    selection.set_autoselect(false);
    let image_signal_list_item_factory = SignalListItemFactory::new();
    image_signal_list_item_factory.connect_setup(clone!(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let button = Button::builder()
            .vexpand(true)
            .hexpand(true)
            .can_shrink(true)
            .sensitive(false)
            .has_tooltip(true)
            .build();
        item.set_child(Some(&button));
    }));

    let folder_path_buffer = TextBuffer::builder().build();
    settings
        .bind("wallpaper-folder", &folder_path_buffer, "text")
        .build();
    let path = folder_path_buffer
        .text(
            &folder_path_buffer.start_iter(),
            &folder_path_buffer.end_iter(),
            false,
        )
        .to_string();
    if path.is_empty() {
        settings
            .set_string("wallpaper-folder", glib::home_dir().to_str().unwrap())
            .unwrap();
    }
    log::trace!("Wallpaper Folder: {}", path);

    let (sender_cache_images, receiver_cache_images): (
        Sender<CacheImageFile>,
        Receiver<CacheImageFile>,
    ) = async_channel::bounded(1);
    let (sender_enable_changer_options_bar, receiver_changer_options_bar): (
        Sender<bool>,
        Receiver<bool>,
    ) = async_channel::bounded(1);

    let image_grid = GridView::builder()
        .model(&selection)
        .factory(&image_signal_list_item_factory)
        .max_columns(30)
        .min_columns(3)
        .focusable(true)
        .single_click_activate(true)
        .focus_on_click(true)
        .build();
    let scrolled_winow = ScrolledWindow::builder()
        .child(&image_grid)
        .valign(Align::Fill)
        .halign(Align::Fill)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .hexpand(true)
        .vexpand(true)
        .build();
    let open_folder_button = Button::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .label("Images Folder")
        .build();
    let folder_path_buffer_copy = folder_path_buffer.clone();
    open_folder_button.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            let dialog = FileDialog::builder()
                .accept_label("Select Folder")
                .title("Wallpapers Folder")
                .build();
            let copy = folder_path_buffer_copy.clone();
            dialog.select_folder(Some(&window), Cancellable::NONE, move |d| {
                if let Ok(f) = d {
                    copy.set_text(f.path().unwrap().canonicalize().unwrap().to_str().unwrap());
                }
            });
        }
    ));

    let monitors = Display::default().unwrap().monitors();
    let monitors = monitors
        .into_iter()
        .filter_map(|o| o.ok())
        .filter_map(|o| o.downcast::<gtk::gdk::Monitor>().ok())
        .filter_map(|m| m.connector())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    debug!("{:?}", monitors);

    let monitors_dropdown =
        DropDown::from_strings(&monitors.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    monitors_dropdown.set_halign(Align::End);
    monitors_dropdown.set_valign(Align::Center);
    settings
        .bind("monitor", &monitors_dropdown, "selected")
        .build();

    let wallpaper_changers_dropdown = get_available_wallpaper_changers()
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>();

    let wallpaper_changers_dropdown = DropDown::from_strings(
        wallpaper_changers_dropdown
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    wallpaper_changers_dropdown.set_halign(Align::End);
    wallpaper_changers_dropdown.set_halign(Align::Center);

    wallpaper_changers_dropdown.connect_selected_notify(clone!(
        #[weak]
        image_list_store,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        monitors_dropdown,
        #[weak]
        settings,
        move |_| {
            WallpaperChangers::killall_changers();
            change_image_button_handlers(
                image_list_store.clone(),
                wallpaper_changers_dropdown.clone(),
                monitors_dropdown,
                &settings,
            );
            hide_unsupported_files(
                image_list_store,
                get_selected_changer(&wallpaper_changers_dropdown, &settings),
            );
        }
    ));

    let previous_wallpapers_text_buffer = TextBuffer::builder().build();
    settings
        .bind("saved-wallpapers", &previous_wallpapers_text_buffer, "text")
        .build();

    let previous_wallpapers_text_buffer = previous_wallpapers_text_buffer.clone();
    image_signal_list_item_factory.connect_bind(clone!(
        #[weak]
        monitors_dropdown,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        settings,
        move |_factory, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let button = item.child().and_downcast::<Button>().unwrap();
            let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let image: Ref<GtkPictureFile> = entry.borrow();
            let path = image.clone().chache_image_file.path;
            button.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
            let previous_wallpapers_text_buffer = previous_wallpapers_text_buffer.clone();
            button.connect_clicked(clone!(
                #[strong]
                path,
                move |_| {
                    let path = path.clone();
                    let selected_monitor = monitors_dropdown
                        .selected_item()
                        .unwrap()
                        .downcast::<StringObject>()
                        .unwrap()
                        .string()
                        .to_string();
                    let selected_changer =
                        get_selected_changer(&wallpaper_changers_dropdown, &settings);
                    match &selected_changer {
                        WallpaperChangers::Hyprpaper => {}
                        WallpaperChangers::Swaybg(mode, color) => {
                            debug!("{mode} {color}")
                        }
                    }
                    let mut previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(
                        &gschema_string_to_string(&settings.string("saved-wallpapers").to_string()),
                    )
                    .unwrap();
                    debug!("Previous wallpapers: {:#?}", previous_wallpapers);
                    match previous_wallpapers
                        .clone()
                        .into_iter()
                        .find(|w| w.monitor == selected_monitor)
                    {
                        Some(mut w) => {
                            w.changer = selected_changer.clone();
                            w.path = path.clone();
                        }
                        None => previous_wallpapers.push(Wallpaper {
                            monitor: selected_monitor.clone(),
                            path: path.clone(),
                            changer: selected_changer.clone(),
                        }),
                    }
                    let saved_wallpapers = string_to_gschema_string(&serde_json::to_string::<Vec<Wallpaper>>(&previous_wallpapers).unwrap());
                    previous_wallpapers_text_buffer.set_text(
                        &saved_wallpapers,
                    );
                    debug!("Stored Text: {}", saved_wallpapers);
                    selected_changer
                        .clone()
                        .change(PathBuf::from(&path.clone()), selected_monitor.clone())
                }
            ));
            button.set_tooltip_text(Some(&image.chache_image_file.name));
            button.set_child(Some(&image.picture));
        }
    ));

    let sort_dropdown = DropDown::from_strings(&["Date", "Name"]);
    sort_dropdown.set_halign(Align::End);
    sort_dropdown.set_valign(Align::Center);
    let invert_sort_switch = Switch::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let invert_sort_switch_label = Text::builder()
        .text("Invert Sort")
        .margin_start(3)
        .margin_top(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .build();

    sort_dropdown.connect_selected_notify(clone!(
        #[weak]
        invert_sort_switch,
        #[weak]
        image_list_store,
        #[weak]
        image_grid,
        move |d| {
            sort_images(d, &invert_sort_switch, &image_list_store, &image_grid);
        }
    ));

    invert_sort_switch.connect_state_notify(clone!(
        #[weak]
        sort_dropdown,
        #[weak]
        image_list_store,
        #[weak]
        image_grid,
        move |s| {
            sort_images(&sort_dropdown, s, &image_list_store, &image_grid);
        }
    ));

    let selected_item = sort_dropdown
        .selected_item()
        .unwrap()
        .downcast::<StringObject>()
        .unwrap()
        .string()
        .to_string();

    settings.bind("sort-by", &sort_dropdown, "selected").build();

    generate_image_files(
        path.clone(),
        sender_cache_images.clone(),
        selected_item.clone(),
        invert_sort_switch.state(),
        sender_enable_changer_options_bar.clone(),
    );

    let changer_specific_options_box = Box::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .hexpand(true)
        .orientation(Orientation::Horizontal)
        .build();

    wallpaper_changers_dropdown.connect_selected_notify(clone!(
        #[weak]
        changer_specific_options_box,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        settings,
        move |_| {
            generate_changer_bar(
                changer_specific_options_box,
                get_selected_changer(&wallpaper_changers_dropdown, &settings),
                settings,
            );
        }
    ));

    settings
        .bind("changer", &wallpaper_changers_dropdown, "selected")
        .build();

    let changer_options_box = Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .sensitive(false)
        .orientation(Orientation::Horizontal)
        .build();
    changer_options_box.append(&monitors_dropdown);
    changer_options_box.append(&open_folder_button);
    changer_options_box.append(&sort_dropdown);
    changer_options_box.append(&invert_sort_switch);
    changer_options_box.append(&invert_sort_switch_label);
    changer_options_box.append(&wallpaper_changers_dropdown);
    changer_options_box.append(&changer_specific_options_box);

    let application_box = Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();
    application_box.append(&scrolled_winow);
    application_box.append(&changer_options_box);

    let selected_item = selected_item.clone();
    folder_path_buffer.connect_changed(clone!(
        #[weak]
        image_list_store,
        #[weak]
        invert_sort_switch,
        #[weak]
        changer_options_box,
        #[strong]
        sender_enable_changer_options_bar,
        move |f| {
            let selected_item = selected_item.clone();
            let sender = sender_cache_images.clone();
            let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
            image_list_store.remove_all();
            let state = invert_sort_switch.state();
            changer_options_box.set_sensitive(false);
            spawn_blocking(clone!(
                #[strong]
                sender_enable_changer_options_bar,
                move || {
                    generate_image_files(
                        path.clone(),
                        sender,
                        selected_item,
                        state,
                        sender_enable_changer_options_bar,
                    );
                }
            ));
        }
    ));

    spawn_future_local(clone!(
        #[weak]
        image_list_store,
        async move {
            while let Ok(image) = receiver_cache_images.recv().await {
                image_list_store.append(&BoxedAnyObject::new(GtkPictureFile {
                    picture: Picture::for_paintable(
                        &Texture::from_bytes(&Bytes::from(&image.image)).unwrap(),
                    ),
                    chache_image_file: image,
                }));
            }
        }
    ));

    let images_loading_spinner = Spinner::builder()
        .spinning(true)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    let images_loading_spinner_label = Label::builder()
        .label("Images are loading. Please wait.")
        .margin_top(12)
        .margin_start(5)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    changer_options_box.append(&images_loading_spinner);
    changer_options_box.append(&images_loading_spinner_label);

    spawn_future_local(clone!(
        #[strong]
        receiver_changer_options_bar,
        #[weak]
        changer_options_box,
        #[weak]
        image_list_store,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        settings,
        #[weak]
        images_loading_spinner,
        #[weak]
        images_loading_spinner_label,
        async move {
            while let Ok(b) = receiver_changer_options_bar.recv().await {
                debug!("Finished loading images");
                changer_options_box.set_sensitive(b);
                if b {
                    debug!("Hiding unsupported images");
                    hide_unsupported_files(
                        image_list_store.clone(),
                        get_selected_changer(&wallpaper_changers_dropdown, &settings),
                    );
                    images_loading_spinner.set_visible(false);
                    images_loading_spinner_label.set_visible(false);
                } else {
                    images_loading_spinner.set_visible(true);
                    images_loading_spinner_label.set_visible(true);
                }
            }
        }
    ));

    generate_changer_bar(
        changer_specific_options_box.clone(),
        get_selected_changer(&wallpaper_changers_dropdown, &settings),
        settings,
    );
    window.set_child(Some(&application_box));
}
