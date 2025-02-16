use crate::{
    cli::Cli,
    common::{
        CacheImageFile, GtkPictureFile, Wallpaper, APP_ID, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
    },
    ui_common::{
        change_image_button_handlers, compare_image_list_items_by_sort_selection_comparitor,
        generate_changer_bar, generate_image_files, get_available_monitors, get_selected_changer,
        gschema_string_to_string, hide_unsupported_files, sort_images, string_to_gschema_string,
        SORT_DROPDOWN_STRINGS,
    },
    wallpaper_changers::{get_available_wallpaper_changers, WallpaperChanger},
};
use async_channel::{Receiver, Sender};
use gettextrs::{gettext, ngettext};
use gtk::{
    self,
    gdk::Texture,
    gio::{spawn_blocking, Cancellable, ListStore, Settings},
    glib::{self, clone, spawn_future_local, BoxedAnyObject, Bytes},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, DropDown, Entry, FileDialog, GridView,
    Label, ListItem, ListScrollFlags, MenuButton, Orientation, Picture, Popover, ProgressBar,
    ScrolledWindow, SignalListItemFactory, SingleSelection, StringObject, Switch, Text, TextBuffer,
};
use log::debug;
use std::{
    cell::{Ref, RefCell},
    path::PathBuf,
    process::Command,
};

#[derive(Clone)]
struct SensitiveWidgetsHelper {
    receiver_changer_options_bar: Receiver<bool>,
    image_list_store: ListStore,
    wallpaper_changers_dropdown: DropDown,
    settings: Settings,
    sort_dropdown: DropDown,
    invert_sort_switch: Switch,
    images_loading_progress_bar: ProgressBar,
    image_grid: GridView,
    changer_specific_options_box: Box,
    removed_images_list_store: ListStore,
    monitors_dropdown: DropDown,
}

pub fn build_ui(app: &Application, args: &Cli) {
    let window = create_application_window(app);
    if get_available_wallpaper_changers().is_empty() {
        create_no_changers_window(&window);
        return;
    }
    let settings = Settings::new(APP_ID);
    let image_list_store = ListStore::new::<BoxedAnyObject>();
    let removed_images_list_store = ListStore::new::<BoxedAnyObject>();
    let folder_path_buffer = create_folder_path_buffer(&settings);
    let path = textbuffer_to_string(&folder_path_buffer);

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

    let open_folder_button = create_open_folder_button(&folder_path_buffer, &window);
    let monitors_dropdown = create_monitors_dropdown(&settings);

    let wallpaper_changers_dropdown = create_wallpaper_changers_dropdown();

    let image_signal_list_item_factory = setup_image_signal_list_item_factory(
        &monitors_dropdown,
        &wallpaper_changers_dropdown,
        &settings,
        args.clone(),
    );

    let image_grid = create_image_grid(&image_signal_list_item_factory, &image_list_store);
    let scrolled_winow = create_image_grid_scrolled_window(&image_grid);

    let sort_dropdown = create_sort_dropdown(&settings);

    let (invert_sort_switch, invert_sort_switch_label) = create_invert_sort_switch(&settings);
    connect_sorting_signals(
        &sort_dropdown,
        &invert_sort_switch,
        &image_list_store,
        &image_grid,
    );

    let selected_sort_method = selected_item_as_string(&sort_dropdown);

    generate_image_files(
        path.clone(),
        sender_cache_images.clone(),
        selected_sort_method.clone(),
        invert_sort_switch.state(),
        sender_enable_changer_options_bar.clone(),
        sender_images_loading_progress_bar.clone(),
    );

    let changer_specific_options_box = create_changer_specific_options_box();

    connect_wallpaper_changers_signals(
        &wallpaper_changers_dropdown,
        &invert_sort_switch,
        &monitors_dropdown,
        &settings,
        &sort_dropdown,
        changer_specific_options_box.clone(),
        (&image_list_store, &removed_images_list_store),
    );

    let image_filter_entry = create_image_filter_entry(
        &settings,
        &image_list_store,
        &monitors_dropdown,
        &sort_dropdown,
        &invert_sort_switch,
        &removed_images_list_store,
	&wallpaper_changers_dropdown
    );

    let options_menu_button =
        create_options_menu_button(&invert_sort_switch, &invert_sort_switch_label);

    let changer_options_box = create_changer_options_box();
    changer_options_box.append(&monitors_dropdown);
    changer_options_box.append(&open_folder_button);
    changer_options_box.append(&sort_dropdown);
    changer_options_box.append(&image_filter_entry);
    changer_options_box.append(&options_menu_button);
    changer_options_box.append(&wallpaper_changers_dropdown);
    changer_options_box.append(&changer_specific_options_box);

    connect_folder_path_buffer_signals(
        &folder_path_buffer,
        &image_list_store,
        &invert_sort_switch,
        (
            sender_enable_changer_options_bar,
            sender_images_loading_progress_bar,
        ),
        &selected_sort_method,
        sender_cache_images,
    );

    let application_box = create_application_box();
    application_box.append(&scrolled_winow);
    application_box.append(&changer_options_box);

    create_cache_image_future(&image_list_store, receiver_cache_images);

    let images_loading_progress_bar = create_images_loading_progress_bar();

    changer_options_box.append(&images_loading_progress_bar);

    let sensitive_widgets_helper = SensitiveWidgetsHelper {
        receiver_changer_options_bar,
        image_list_store,
        wallpaper_changers_dropdown: wallpaper_changers_dropdown.clone(),
        settings: settings.clone(),
        sort_dropdown,
        invert_sort_switch,
        images_loading_progress_bar: images_loading_progress_bar.clone(),
        image_grid,
        changer_specific_options_box: changer_specific_options_box.clone(),
        removed_images_list_store,
        monitors_dropdown,
    };
    create_disable_ui_future(sensitive_widgets_helper);

    create_progress_image_loading_progress_bar_future(
        receiver_images_loading_progress_bar,
        images_loading_progress_bar,
    );

    generate_changer_bar(
        &changer_specific_options_box,
        &get_selected_changer(&wallpaper_changers_dropdown, &settings),
        settings,
    );
    window.set_child(Some(&application_box));
}

fn setup_image_signal_list_item_factory(
    monitors_dropdown: &DropDown,
    wallpaper_changers_dropdown: &DropDown,
    settings: &Settings,
    args: Cli,
) -> SignalListItemFactory {
    let image_signal_list_item_factory = SignalListItemFactory::new();

    let previous_wallpapers_text_buffer = TextBuffer::builder().build();
    settings
        .bind("saved-wallpapers", &previous_wallpapers_text_buffer, "text")
        .build();
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

    bind_image_list_item_factory(
        &image_signal_list_item_factory,
        monitors_dropdown,
        wallpaper_changers_dropdown,
        settings,
        args,
        previous_wallpapers_text_buffer,
    );
    image_signal_list_item_factory
}

fn bind_image_list_item_factory(
    image_signal_list_item_factory: &SignalListItemFactory,
    monitors_dropdown: &DropDown,
    wallpaper_changers_dropdown: &DropDown,
    settings: &Settings,
    args: Cli,
    previous_wallpapers_text_buffer: TextBuffer,
) {
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
                                });
                            }
                            for wallpaper in &mut previous_wallpapers {
                                if wallpaper.monitor == selected_monitor {
                                    wallpaper.path.clone_from(&path);
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
                            execute_external_script(&args, &path, &selected_monitor, &settings);
                        }
                    )))),
            };
            button.set_tooltip_text(Some(&image.chache_image_file.name));
            button.set_child(Some(&image.picture));
        }
    ));
}

fn execute_external_script(args: &Cli, path: &str, selected_monitor: &str, settings: &Settings) {
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
                log::debug!("External Script Executed Successfully");
            }
            Err(e) => {
                log::warn!("External Script Failed to Execute: {e}");
            }
        }
    }
}

#[must_use]
pub fn create_open_folder_button(
    folder_path_buffer: &TextBuffer,
    window: &ApplicationWindow,
) -> Button {
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
    open_folder_button
}

fn create_monitors_dropdown(settings: &Settings) -> DropDown {
    let mut monitors = get_available_monitors();
    monitors.insert(0, gettext("All"));
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

    let monitors_dropdown = DropDown::from_strings(
        &monitors
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<_>>(),
    );
    monitors_dropdown.set_halign(Align::End);
    monitors_dropdown.set_valign(Align::Center);
    monitors_dropdown.set_margin_top(12);
    monitors_dropdown.set_margin_start(12);
    monitors_dropdown.set_margin_bottom(12);
    monitors_dropdown.set_margin_end(12);
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
    monitors_dropdown
}

fn create_wallpaper_changers_dropdown() -> DropDown {
    let wallpaper_changers_dropdown = get_available_wallpaper_changers()
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>();
    let wallpaper_changers_dropdown = DropDown::from_strings(
        wallpaper_changers_dropdown
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
    );

    wallpaper_changers_dropdown.set_halign(Align::End);
    wallpaper_changers_dropdown.set_halign(Align::Center);
    wallpaper_changers_dropdown.set_margin_top(12);
    wallpaper_changers_dropdown.set_margin_start(12);
    wallpaper_changers_dropdown.set_margin_bottom(12);
    wallpaper_changers_dropdown.set_margin_end(12);
    wallpaper_changers_dropdown
}

fn create_image_grid(
    image_signal_list_item_factory: &SignalListItemFactory,
    image_list_store: &ListStore,
) -> GridView {
    let selection = SingleSelection::builder()
        .model(&image_list_store.clone())
        .autoselect(false)
        .build();
    GridView::builder()
        .model(&selection)
        .factory(image_signal_list_item_factory)
        .max_columns(30)
        .min_columns(3)
        .focusable(true)
        .single_click_activate(true)
        .focus_on_click(true)
        .build()
}

fn create_folder_path_buffer(settings: &Settings) -> TextBuffer {
    let folder_path_buffer = TextBuffer::builder().build();
    settings
        .bind("wallpaper-folder", &folder_path_buffer, "text")
        .build();
    folder_path_buffer
}

fn create_image_grid_scrolled_window(image_grid: &GridView) -> ScrolledWindow {
    ScrolledWindow::builder()
        .child(image_grid)
        .valign(Align::Fill)
        .halign(Align::Fill)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .hexpand(true)
        .vexpand(true)
        .build()
}

fn create_sort_dropdown(settings: &Settings) -> DropDown {
    let strings = SORT_DROPDOWN_STRINGS
        .into_iter()
        .map(gettext)
        .collect::<Vec<_>>();
    let strings = strings.iter().map(String::as_str).collect::<Vec<_>>();
    let sort_dropdown = DropDown::from_strings(&strings);
    sort_dropdown.set_halign(Align::End);
    sort_dropdown.set_valign(Align::Center);
    sort_dropdown.set_margin_top(12);
    sort_dropdown.set_margin_start(12);
    sort_dropdown.set_margin_bottom(12);
    sort_dropdown.set_margin_end(12);
    settings.bind("sort-by", &sort_dropdown, "selected").build();
    sort_dropdown
}

fn create_invert_sort_switch(settings: &Settings) -> (Switch, Text) {
    let switch = Switch::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let text = Text::builder()
        .text(gettext("Invert Sort"))
        .margin_start(3)
        .margin_top(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    settings.bind("invert-sort", &switch, "active").build();
    (switch, text)
}

fn connect_sorting_signals(
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    image_list_store: &ListStore,
    image_grid: &GridView,
) {
    sort_dropdown.connect_selected_notify(clone!(
        #[strong]
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
}

fn connect_wallpaper_changers_signals(
    wallpaper_changers_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    monitors_dropdown: &DropDown,
    settings: &Settings,
    sort_dropdown: &DropDown,
    changer_specific_options_box: Box,
    (image_list_store, removed_images_list_store): (&ListStore, &ListStore),
) {
    wallpaper_changers_dropdown.connect_selected_item_notify(clone!(
        #[weak]
        image_list_store,
        #[weak]
        monitors_dropdown,
        #[weak]
        settings,
        #[weak]
        sort_dropdown,
        #[strong]
        invert_sort_switch,
        #[strong]
        removed_images_list_store,
        move |w| {
            change_image_button_handlers(&image_list_store, w, &monitors_dropdown, &settings);
            hide_unsupported_files(
                &image_list_store,
                &get_selected_changer(w, &settings),
                &removed_images_list_store,
                &sort_dropdown,
                &invert_sort_switch,
                settings.string("image-filter").as_ref(),
            );
            generate_changer_bar(
                &changer_specific_options_box,
                &get_selected_changer(w, &settings),
                settings,
            );
        }
    ));
    settings
        .bind("changer", wallpaper_changers_dropdown, "selected")
        .build();
}

fn create_options_menu_button(
    invert_sort_switch: &Switch,
    invert_sort_switch_label: &Text,
) -> MenuButton {
    let options_box = Box::builder().orientation(Orientation::Vertical).build();
    let sort_invert_box = Box::builder().orientation(Orientation::Horizontal).build();
    sort_invert_box.append(invert_sort_switch_label);
    sort_invert_box.append(invert_sort_switch);
    options_box.append(&sort_invert_box);

    let options_popover_menu = Popover::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .child(&options_box)
        .build();
    MenuButton::builder()
        .popover(&options_popover_menu)
        .halign(Align::Start)
        .valign(Align::Center)
        .margin_start(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_end(12)
        .label(gettext("Options"))
        .build()
}

fn create_changer_options_box() -> Box {
    Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .orientation(Orientation::Horizontal)
        .build()
}

fn connect_folder_path_buffer_signals(
    folder_path_buffer: &TextBuffer,
    image_list_store: &ListStore,
    invert_sort_switch: &Switch,
    (sender_enable_changer_options_bar, sender_images_loading_progress_bar): (
        Sender<bool>,
        Sender<f64>,
    ),
    selected_sort_method: &str,
    sender_cache_images: Sender<CacheImageFile>,
) {
    let selected_sort_method = selected_sort_method.to_string();
    folder_path_buffer.connect_changed(clone!(
        #[weak]
        image_list_store,
        #[strong]
        invert_sort_switch,
        #[strong]
        sender_enable_changer_options_bar,
        #[strong]
        sender_images_loading_progress_bar,
        #[strong]
        selected_sort_method,
        move |f| {
            let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
            image_list_store.remove_all();
            let state = invert_sort_switch.state();
            let selected_sort_method = selected_sort_method.to_string();
            spawn_blocking(clone!(
                #[strong]
                sender_enable_changer_options_bar,
                #[strong]
                sender_images_loading_progress_bar,
                #[strong]
                selected_sort_method,
                #[strong]
                sender_cache_images,
                move || {
                    generate_image_files(
                        path.clone(),
                        sender_cache_images,
                        selected_sort_method,
                        state,
                        sender_enable_changer_options_bar,
                        sender_images_loading_progress_bar,
                    );
                }
            ));
        }
    ));
}

fn create_application_box() -> Box {
    Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build()
}

fn selected_item_as_string(dropdown: &DropDown) -> String {
    dropdown
        .selected_item()
        .unwrap()
        .downcast::<StringObject>()
        .unwrap()
        .string()
        .to_string()
}

fn create_changer_specific_options_box() -> Box {
    Box::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .hexpand(true)
        .orientation(Orientation::Horizontal)
        .build()
}

fn create_cache_image_future(
    image_list_store: &ListStore,
    receiver_cache_images: Receiver<CacheImageFile>,
) {
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
}

fn create_images_loading_progress_bar() -> ProgressBar {
    ProgressBar::builder()
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
        .build()
}

fn create_disable_ui_future(sensitive_widgets_helper: SensitiveWidgetsHelper) {
    spawn_future_local(clone!(async move {
        while let Ok(b) = sensitive_widgets_helper
            .receiver_changer_options_bar
            .recv()
            .await
        {
            debug!("{}", gettext("Finished loading images"));
            sensitive_widgets_helper
                .images_loading_progress_bar
                .set_visible(!b);
            sensitive_widgets_helper.monitors_dropdown.set_sensitive(b);
            sensitive_widgets_helper
                .clone()
                .sort_dropdown
                .set_sensitive(b);
            sensitive_widgets_helper
                .clone()
                .invert_sort_switch
                .set_sensitive(b);
            sensitive_widgets_helper
                .wallpaper_changers_dropdown
                .set_sensitive(b);
            sensitive_widgets_helper
                .changer_specific_options_box
                .set_sensitive(b);
            sensitive_widgets_helper.image_grid.set_sensitive(b);
            if b {
                debug!("{}", gettext("Hiding unsupported images"));
                hide_unsupported_files(
                    &sensitive_widgets_helper.clone().image_list_store,
                    &get_selected_changer(
                        &sensitive_widgets_helper.wallpaper_changers_dropdown,
                        &sensitive_widgets_helper.clone().settings,
                    ),
                    &sensitive_widgets_helper.clone().removed_images_list_store,
                    &sensitive_widgets_helper.clone().sort_dropdown,
                    &sensitive_widgets_helper.clone().invert_sort_switch,
                    sensitive_widgets_helper
                        .settings
                        .string("image-filter")
                        .as_ref(),
                );
                sensitive_widgets_helper.image_list_store.sort(
                    compare_image_list_items_by_sort_selection_comparitor(
                        sensitive_widgets_helper.sort_dropdown.clone(),
                        sensitive_widgets_helper.invert_sort_switch.clone(),
                    ),
                );
                sensitive_widgets_helper
                    .image_grid
                    .scroll_to(0, ListScrollFlags::NONE, None);
            }
        }
    }));
}

fn create_progress_image_loading_progress_bar_future(
    receiver_images_loading_progress_bar: Receiver<f64>,
    images_loading_progress_bar: ProgressBar,
) {
    spawn_future_local(clone!(async move {
        while let Ok(f) = receiver_images_loading_progress_bar.recv().await {
            images_loading_progress_bar.set_fraction(f);
        }
    }));
}

fn create_application_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Watering")
        .build();
    window.set_default_size(1024, 600);
    window.present();
    window
}

fn textbuffer_to_string(text_buffer: &TextBuffer) -> String {
    text_buffer
        .text(&text_buffer.start_iter(), &text_buffer.end_iter(), false)
        .to_string()
}

fn create_no_changers_window(window: &ApplicationWindow) {
    let application_box = create_application_box();
    let text_box = Box::builder()
        .halign(Align::Center)
        .valign(Align::Center)
        .orientation(Orientation::Horizontal)
        .build();
    let confirm_button = Button::builder()
        .label(gettext("Ok"))
        .vexpand(true)
        .hexpand(true)
        .can_shrink(true)
        .has_tooltip(true)
        .tooltip_text(gettext("Close waytrogen"))
        .valign(Align::End)
        .halign(Align::Center)
        .hexpand(true)
        .build();
    let error_message_label = Label::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .label(gettext(
            "No wallpaper changers detected.\n
Please install one or more of the following:\n\n
- Hyprpaper\n
- Swaybg\n
- Mpvpaper\n
- SWWW\n\n
If waytrogen continues failing to detect an installed changer,\n
please feel free open issue on the GitHub repository:\n
https://github.com/nikolaizombie1/waytrogen/issues",
        ))
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    confirm_button.connect_clicked(clone!(
        #[strong]
        window,
        move |_| {
            window.close();
        }
    ));
    text_box.append(&error_message_label);
    application_box.append(&text_box);
    application_box.append(&confirm_button);
    window.set_child(Some(&application_box));
}

fn create_image_filter_entry(
    settings: &Settings,
    image_list_store: &ListStore,
    monitors_dropdown: &DropDown,
    sort_dropdown: &DropDown,
    invert_sort_switch: &Switch,
    removed_images_list_store: &ListStore,
    wallpaper_changers_dropdown: &DropDown
) -> Entry {
    let entry = Entry::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .placeholder_text(gettext("Find images"))
        .has_tooltip(true)
        .tooltip_text(gettext(
            "Filter wallpapers based on the name. Fuzzy matching the name.",
        ))
        .build();
    settings
        .bind("image-filter", &entry.buffer(), "text")
        .build();
    entry.connect_changed(clone!(
        #[strong]
        image_list_store,
        #[strong]
        monitors_dropdown,
        #[strong]
        settings,
        #[strong]
        sort_dropdown,
        #[strong]
        invert_sort_switch,
        #[strong]
        removed_images_list_store,
	#[strong]
	wallpaper_changers_dropdown,
        move |e| {
            change_image_button_handlers(&image_list_store, &wallpaper_changers_dropdown, &monitors_dropdown, &settings);
            hide_unsupported_files(
                &image_list_store,
                &get_selected_changer(&wallpaper_changers_dropdown, &settings),
                &removed_images_list_store,
                &sort_dropdown,
                &invert_sort_switch,
                e.text().as_ref()
            );
        }
    ));
    entry
}
