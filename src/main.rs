use std::{
    cell::{Ref, RefCell},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

use async_channel::{Receiver, Sender};
use clap::Parser;
use gtk::{
    self,
    gdk::{Display, Texture},
    gio::{spawn_blocking, Cancellable, ListStore, Settings},
    glib::{self, clone, spawn_future_local, BoxedAnyObject, Bytes},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, DropDown, FileDialog, GridView, ListItem,
    Orientation, Picture, ProgressBar, ScrolledWindow, SignalListItemFactory, SingleSelection,
    StringObject, Switch, Text, TextBuffer,
};
use log::debug;
use rand::Rng;
use waytrogen::{
    common::{
        CacheImageFile, Cli, GtkPictureFile, Wallpaper, APP_ID, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
    },
    ui_common::{
        change_image_button_handlers, generate_changer_bar, generate_image_files,
        get_available_wallpaper_changers, get_selected_changer, gschema_string_to_string,
        hide_unsupported_files, sort_images, string_to_gschema_string,
    },
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
};

use gettextrs::*;

fn main() -> glib::ExitCode {
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbosity as usize)
        .init()
        .unwrap();
    // Create a new application

    let settings = Settings::new(APP_ID);
    if args.restore {
        WallpaperChangers::killall_changers();
        let previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(
            &gschema_string_to_string(settings.string("saved-wallpapers").as_ref()),
        )
        .unwrap();
        for wallpaper in previous_wallpapers {
            debug!("Restoring: {:?}", wallpaper);
            wallpaper.clone().changer.change(
                PathBuf::from(wallpaper.clone().path),
                wallpaper.clone().monitor,
            );
            match wallpaper.clone().changer {
                WallpaperChangers::Hyprpaper => {
                    thread::sleep(Duration::from_millis(1000));
                }
                WallpaperChangers::Swaybg(_, _) => {}
                WallpaperChangers::MpvPaper(_, _, _) => {}
                WallpaperChangers::Swww(_, _, _, _, _, _, _, _, _, _, _, _) => {}
            }
        }
        glib::ExitCode::SUCCESS
    } else if args.list_current_wallpapers {
        println!(
            "{}",
            gschema_string_to_string(&settings.string("saved-wallpapers"))
        );
        glib::ExitCode::SUCCESS
    } else if args.random {
        WallpaperChangers::killall_changers();
        let previous_wallpapers = serde_json::from_str::<Vec<Wallpaper>>(
            &gschema_string_to_string(settings.string("saved-wallpapers").as_ref()),
        )
        .unwrap();
        let wallpaper = previous_wallpapers[0].clone();
        let path = Path::new(&wallpaper.path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let files = walkdir::WalkDir::new(path)
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
        previous_wallpapers.iter().for_each(|w| {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..files.len());
            log::debug!("{index}");
            w.changer
                .clone()
                .change(files[index].clone(), w.monitor.clone());
        });
        glib::ExitCode::SUCCESS
    } else {
        let app = Application::builder().application_id(APP_ID).build();
        textdomain("waytrogen").unwrap();
        bind_textdomain_codeset("waytrogen", "UTF-8").unwrap();
        bindtextdomain("waytrogen", "/usr/share/locale/").unwrap();

        app.connect_activate(move |app| {
            build_ui(app, args.clone());
        });

        let empty: Vec<String> = vec![];
        // Run the application
        app.run_with_args(&empty)
    }
}

fn build_ui(app: &Application, args: Cli) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Watering")
        .build();

    window.present();

    let settings = Settings::new(APP_ID);

    let image_list_store = ListStore::new::<BoxedAnyObject>();
    let removed_images_list_store = ListStore::new::<BoxedAnyObject>();

    let selection = SingleSelection::builder()
        .model(&image_list_store.clone())
        .autoselect(false)
        .build();
    let image_signal_list_item_factory = SignalListItemFactory::new();
    image_signal_list_item_factory.connect_setup(clone!(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let button = Button::builder()
            .vexpand(true)
            .hexpand(true)
            .can_shrink(true)
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

    log::trace!("{}: {}", gettext("Wallpaper Folder"), path);

    let (sender_cache_images, receiver_cache_images): (
        Sender<CacheImageFile>,
        Receiver<CacheImageFile>,
    ) = async_channel::bounded(1);
    let (sender_enable_changer_options_bar, receiver_changer_options_bar): (
        Sender<bool>,
        Receiver<bool>,
    ) = async_channel::bounded(1);
    let (sender_images_loading_progress_bar, receiver_images_loading_progress_bar): (
        Sender<f64>,
        Receiver<f64>,
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
        .label(ngettext("Image Folder", "Images Folder", 2))
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
    let selected_monitor_text_buffer = TextBuffer::builder().build();
    debug!("{:?}", monitors);
    settings
        .bind(
            "selected-monitor-item",
            &selected_monitor_text_buffer,
            "text",
        )
        .build();
    selected_monitor_text_buffer.set_text(settings.string("selected-monitor-item").as_ref());

    let monitors_dropdown =
        DropDown::from_strings(&monitors.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    monitors_dropdown.set_halign(Align::End);
    monitors_dropdown.set_valign(Align::Center);
    settings
        .bind("monitor", &monitors_dropdown, "selected")
        .build();
    monitors_dropdown.connect_selected_notify(clone!(
        #[weak]
        settings,
        move |i| {
            let selected_monitor = i
                .selected_item()
                .and_downcast::<StringObject>()
                .unwrap()
                .string()
                .to_string();
            selected_monitor_text_buffer.set_text(&selected_monitor);
            settings
                .bind(
                    "selected-monitor-item",
                    &selected_monitor_text_buffer,
                    "text",
                )
                .build();
        }
    ));

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

    let previous_wallpapers_text_buffer = TextBuffer::builder().build();
    settings
        .bind("saved-wallpapers", &previous_wallpapers_text_buffer, "text")
        .build();

    let previous_wallpapers_text_buffer = previous_wallpapers_text_buffer.clone();
    let args = args.clone();
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
            let path = &image.chache_image_file.path;
            let args = args.clone();
            button.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
            let previous_wallpapers_text_buffer = previous_wallpapers_text_buffer.clone();
            let handler = image.button_signal_handler.take();
            match handler {
                Some(h) => image.button_signal_handler.replace(Some(h)),
                None => image
                    .button_signal_handler
                    .replace(Some(button.connect_clicked(clone!(
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
                            let mut previous_wallpapers =
                                serde_json::from_str::<Vec<Wallpaper>>(&gschema_string_to_string(
                                    settings.string("saved-wallpapers").as_ref(),
                                ))
                                .unwrap();
                            let mut new_monitor_wallpapers: Vec<Wallpaper> = vec![];
                            if !previous_wallpapers
                                .iter()
                                .any(|w| w.monitor == selected_monitor.clone())
                            {
                                new_monitor_wallpapers.push(Wallpaper {
                                    monitor: selected_monitor.clone(),
                                    path: path.clone(),
                                    changer: selected_changer.clone(),
                                })
                            }
                            for wallpaper in &mut previous_wallpapers {
                                if wallpaper.monitor == selected_monitor {
                                    wallpaper.path = path.clone();
                                    wallpaper.changer = selected_changer.clone();
                                }
                            }
                            previous_wallpapers.append(&mut new_monitor_wallpapers);
                            let previous_wallpapers = previous_wallpapers
                                .clone()
                                .into_iter()
                                .map(|w| Wallpaper {
                                    monitor: w.monitor,
                                    path: w.path,
                                    changer: selected_changer.clone(),
                                })
                                .collect::<Vec<_>>();
                            debug!(
                                "{}: {:#?}",
                                gettext("Saved wallpapers"),
                                previous_wallpapers
                            );
                            let saved_wallpapers = string_to_gschema_string(
                                &serde_json::to_string::<Vec<Wallpaper>>(&previous_wallpapers)
                                    .unwrap(),
                            );
                            previous_wallpapers_text_buffer.set_text(&saved_wallpapers);
                            debug!("{}: {}", gettext("Stored Text"), saved_wallpapers);
                            selected_changer
                                .clone()
                                .change(PathBuf::from(&path.clone()), selected_monitor.clone());
                            if args.external_script != *"" {
                                match Command::new(args.external_script.clone())
                                    .arg(selected_monitor)
                                    .arg(path)
                                    .arg(gschema_string_to_string(&gschema_string_to_string(
                                        settings.string("saved-wallpapers").as_ref(),
                                    )))
                                    .spawn()
                                {
                                    Ok(_) => {
                                        log::debug!("External Script Executed Successfully")
                                    }
                                    Err(e) => {
                                        log::warn!("External Script Failed to Execute: {e}")
                                    }
                                }
                            }
                        }
                    )))),
            };
            button.set_tooltip_text(Some(&image.chache_image_file.name));
            button.set_child(Some(&image.picture));
        }
    ));

    let sort_dropdown = DropDown::from_strings(&[&gettext("Date"), &gettext("Name")]);
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
        .text(gettext("Invert Sort"))
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

    let removed_images_list_store_copy = removed_images_list_store.clone();
    wallpaper_changers_dropdown.connect_selected_item_notify(clone!(
        #[weak]
        image_list_store,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        monitors_dropdown,
        #[weak]
        settings,
        #[weak]
        sort_dropdown,
        #[weak]
        invert_sort_switch,
        move |_| {
            change_image_button_handlers(
                image_list_store.clone(),
                wallpaper_changers_dropdown.clone(),
                monitors_dropdown,
                &settings,
            );
            hide_unsupported_files(
                image_list_store,
                get_selected_changer(&wallpaper_changers_dropdown, &settings),
                &removed_images_list_store_copy,
                &sort_dropdown,
                &invert_sort_switch,
            );
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
        sender_images_loading_progress_bar.clone(),
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

    let sender_images_loading_progress_bar_copy = sender_images_loading_progress_bar.clone();
    folder_path_buffer.connect_changed(clone!(
        #[weak]
        image_list_store,
        #[weak]
        invert_sort_switch,
        #[strong]
        sender_enable_changer_options_bar,
        move |f| {
            let selected_item = selected_item.clone();
            let sender = sender_cache_images.clone();
            let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
            image_list_store.remove_all();
            let state = invert_sort_switch.state();
            let sender_images_loading_progress_bar_copy =
                sender_images_loading_progress_bar_copy.clone();
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
                        sender_images_loading_progress_bar_copy,
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
                    button_signal_handler: RefCell::new(None),
                }));
            }
        }
    ));

    let images_loading_progress_bar = ProgressBar::builder()
        .opacity(1.0)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .text(gettext("Images are loading, please wait"))
        .show_text(true)
        .visible(true)
        .sensitive(true)
        .build();

    changer_options_box.append(&images_loading_progress_bar);

    let removed_images_list_store = removed_images_list_store.clone();
    spawn_future_local(clone!(
        #[strong]
        receiver_changer_options_bar,
        #[weak]
        image_list_store,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        settings,
        #[weak]
        sort_dropdown,
        #[weak]
        invert_sort_switch,
        #[weak]
        images_loading_progress_bar,
        #[weak]
        image_grid,
        #[weak]
        changer_specific_options_box,
        async move {
            while let Ok(b) = receiver_changer_options_bar.recv().await {
                debug!("{}", gettext("Finished loading images"));
                images_loading_progress_bar.set_visible(!b);
                monitors_dropdown.set_sensitive(b);
                sort_dropdown.set_sensitive(b);
                invert_sort_switch.set_sensitive(b);
                invert_sort_switch_label.set_sensitive(b);
                wallpaper_changers_dropdown.set_sensitive(b);
                changer_specific_options_box.set_sensitive(b);

                image_grid.set_sensitive(b);
                if b {
                    debug!("{}", gettext("Hiding unsupported images"));
                    hide_unsupported_files(
                        image_list_store.clone(),
                        get_selected_changer(&wallpaper_changers_dropdown, &settings),
                        &removed_images_list_store,
                        &sort_dropdown,
                        &invert_sort_switch,
                    );
                }
            }
        }
    ));

    spawn_future_local(clone!(async move {
        while let Ok(f) = receiver_images_loading_progress_bar.recv().await {
            images_loading_progress_bar.set_fraction(f);
        }
    }));

    generate_changer_bar(
        changer_specific_options_box.clone(),
        get_selected_changer(&wallpaper_changers_dropdown, &settings),
        settings,
    );
    window.set_size_request(800, 800);
    window.set_child(Some(&application_box));
}
