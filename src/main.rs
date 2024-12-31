use std::{cell::Ref, path::Path};

use async_channel::{Receiver, Sender};
use gtk::{
    self, gdk::{Display, Texture}, gio::{spawn_blocking, Cancellable, ListStore, Settings}, glib::{self, clone, spawn_future_local, BoxedAnyObject, Bytes}, prelude::*, Align, Application, ApplicationWindow, Box, Button, ColorDialogButton, DropDown, FileDialog, GridView, ListItem, ListScrollFlags, Orientation, Picture, ScrolledWindow, SignalListItemFactory, SingleSelection, StringObject, Switch, Text, TextBuffer
};
use log::{debug, error};
use strum::IntoEnumIterator;
use waytrogen::{
    common::{GtkImageFile, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH},
    database::DatabaseConnection,
    wallpaper_changers::{WallpaperChanger, WallpaperChangers, SwaybgModes},
};
use which::which;

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
    if path == "" {
        settings
            .set_string(
                "wallpaper-folder",
                &format!("{}", glib::home_dir().to_str().unwrap()),
            )
            .unwrap();
    }
    log::trace!("Wallpaper Folder: {}", path);

    let (sender, receiver): (Sender<GtkImageFile>, Receiver<GtkImageFile>) =
        async_channel::bounded(1);

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
    monitors_dropdown.set_halign(Align::Start);
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


    wallpaper_changers_dropdown.connect_selected_notify(clone!(
        #[weak]
        image_list_store,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        monitors_dropdown,
        move |_| {
            change_image_button_handlers(
                image_list_store,
                wallpaper_changers_dropdown,
                monitors_dropdown,
            );
        }
    ));

    image_signal_list_item_factory.connect_bind(clone!(
        #[weak]
        monitors_dropdown,
        #[weak]
        wallpaper_changers_dropdown,
        move |_factory, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let button = item.child().and_downcast::<Button>().unwrap();
            let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let image: Ref<GtkImageFile> = entry.borrow();
            let path = image.clone().path;
            button.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
            button.connect_clicked(move |_| {
                let selected_monitor = monitors_dropdown
                    .selected_item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string()
                    .to_string();
                let selected_changer = wallpaper_changers_dropdown
                    .selected_item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string()
                    .to_string()
                    .parse::<WallpaperChangers>()
                    .unwrap();
                match selected_changer.change(&Path::new(&path), &selected_monitor) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to change wallpaper: {}", e)
                    }
                }
            });
            button.set_tooltip_text(Some(&image.name));
            button.set_child(Some(&Picture::for_paintable(
                &Texture::from_bytes(&Bytes::from(&image.image)).unwrap(),
            )));
        }
    ));

    let sort_dropdown = DropDown::from_strings(&["Date", "Name"]);
    let invert_sort_switch = Switch::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let invert_sort_switch_label = Text::builder()
        .text("Invert Sort")
        .margin_start(3)
        .margin_top(12)
        .margin_bottom(12)
        .margin_end(12)
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
        sender.clone(),
        selected_item.clone(),
        invert_sort_switch.state(),
    );

    let changer_specific_options_box = Box::builder().halign(Align::Fill).orientation(Orientation::Horizontal).build();

    wallpaper_changers_dropdown.connect_selected_notify(clone!(
        #[weak]
        changer_specific_options_box,
        #[weak]
        wallpaper_changers_dropdown,
        #[weak]
        settings,
        move |_| {
        generate_changer_bar(changer_specific_options_box, get_selected_changer(&wallpaper_changers_dropdown, &settings), settings);
    }));

    settings
        .bind("changer", &wallpaper_changers_dropdown, "selected")
        .build();

    let changer_options_box = Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .halign(Align::Fill)
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
        move |f| {
            let selected_item = selected_item.clone();
            let sender = sender.clone();
            let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
            image_list_store.remove_all();
            let state = invert_sort_switch.state();
            spawn_blocking(move || {
                generate_image_files(path.clone(), sender, selected_item, state);
            });
        }
    ));

    spawn_future_local(clone!(
        #[weak]
        image_list_store,
        async move {
            while let Ok(image) = receiver.recv().await {
                image_list_store.append(&BoxedAnyObject::new(image));
            }
        }
    ));



    generate_changer_bar(changer_specific_options_box.clone(), get_selected_changer(&wallpaper_changers_dropdown, &settings), settings);
    window.set_child(Some(&application_box));
}

fn sort_images(
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
                let image1: Ref<GtkImageFile> = image1.borrow();
                let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
                let image2: Ref<GtkImageFile> = image2.borrow();
                if invert_sort_switch.state() {
                    image1.name.partial_cmp(&image2.name).unwrap()
                } else {
                    image2.name.partial_cmp(&image1.name).unwrap()
                }
            });
        }
        "Date" => {
            image_list_store.sort(|img1, img2| {
                let image1 = img1.downcast_ref::<BoxedAnyObject>().unwrap();
                let image1: Ref<GtkImageFile> = image1.borrow();
                let image2 = img2.downcast_ref::<BoxedAnyObject>().unwrap();
                let image2: Ref<GtkImageFile> = image2.borrow();
                if invert_sort_switch.state() {
                    image1.date.partial_cmp(&image2.date).unwrap()
                } else {
                    image2.date.partial_cmp(&image1.date).unwrap()
                }
            });
        }
        _ => {}
    }
    image_grid.scroll_to(0, ListScrollFlags::FOCUS, None);
}

fn generate_image_files(
    path: String,
    sender: Sender<GtkImageFile>,
    sort_dropdown: String,
    invert_sort_switch_state: bool,
) {
    spawn_blocking(move || {
        let mut files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .collect::<Vec<_>>();

        match &sort_dropdown.to_lowercase()[..] {
            "name" => {
                files.sort_by(|f1, f2| {
                    if invert_sort_switch_state {
                        f2.file_name().partial_cmp(f1.file_name()).unwrap()
                    } else {
                        f1.file_name().partial_cmp(f2.file_name()).unwrap()
                    }
                });
            }
            "date" => {
                files.sort_by(|f1, f2| {
                    if invert_sort_switch_state {
                        f2.metadata()
                            .unwrap()
                            .created()
                            .unwrap()
                            .partial_cmp(&f1.metadata().unwrap().created().unwrap())
                            .unwrap()
                    } else {
                        f1.metadata()
                            .unwrap()
                            .created()
                            .unwrap()
                            .partial_cmp(&f2.metadata().unwrap().created().unwrap())
                            .unwrap()
                    }
                });
            }
            _ => {}
        }

        for file in files {
            match DatabaseConnection::check_cache(file.path()) {
                Ok(i) => sender.send_blocking(i).expect("The channel must be open"),
                Err(_) => {}
            }

        }
    });
}

fn get_available_wallpaper_changers() -> Vec<WallpaperChangers> {
    let mut available_changers = vec![];
    for changer in WallpaperChangers::iter() {
        if let Ok(_) = which(changer.to_string().to_lowercase()) {
            available_changers.push(changer);
        }
    }
    available_changers
}

fn change_image_button_handlers(
    image_list_store: ListStore,
    wallpaper_changers_dropdown: DropDown,
    selected_monitor_dropdown: DropDown,
) {
    image_list_store
        .into_iter()
        .filter_map(|o| o.ok())
        .filter_map(|o| o.downcast::<ListItem>().ok())
        .for_each(|li| {
            let entry = li.item().and_downcast::<BoxedAnyObject>().unwrap();
            let image: Ref<GtkImageFile> = entry.borrow();
            let selected_monitor = selected_monitor_dropdown
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string()
                .to_string();
            let selected_changer = wallpaper_changers_dropdown
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string()
                .to_string()
                .parse::<WallpaperChangers>()
                .unwrap();
            match selected_changer.change(Path::new(&image.path), &selected_monitor) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to change wallpaper: {}", e)
                }
            }
        });
}

static  SWAY_BG_MODES: &'static[&str] = &["stretch", "fit", "fill", "center", "tile", "solid_color"];

fn generate_changer_bar(changer_specific_options_box: Box, selected_changer: WallpaperChangers, settings: Settings) {
    while changer_specific_options_box.first_child().is_some() {
        changer_specific_options_box.remove(&changer_specific_options_box.first_child().unwrap());
    }
    match selected_changer {
        WallpaperChangers::Hyprpaper => {},
        WallpaperChangers::Swaybg(_,_) => {
            let dropdown = DropDown::from_strings(&["stretch", "fit", "fill", "center", "tile", "solid_color"]);
            dropdown.set_halign(Align::End);
            dropdown.set_valign(Align::Center);
            changer_specific_options_box.append(&dropdown);
            let color_picker = ColorDialogButton::builder().halign(Align::End).valign(Align::Center).build();
            changer_specific_options_box.append(&dropdown);
            changer_specific_options_box.append(&color_picker);
            settings.bind("swaybg-mode", &dropdown, "selected").build();
            settings.bind("swaybg-color", &color_picker, "selected").build();
        },
    }
}

fn get_selected_changer(wallpaper_changers_dropdown: &DropDown, settings: &Settings, ) -> WallpaperChangers {
   let selected_item = wallpaper_changers_dropdown.selected_item().unwrap().downcast::<StringObject>() .unwrap().string().to_string().to_lowercase();
   match &selected_item[..] {
        "hyprpaper" => WallpaperChangers::Hyprpaper,
        "swaybg" => {
            let mode =  SWAY_BG_MODES[settings.int("swaybg-mode") as usize].parse::<SwaybgModes>().unwrap();
            let rgb = settings.string("swaybg-color").to_string();
            WallpaperChangers::Swaybg(mode, rgb)
        },
        _ => WallpaperChangers::Hyprpaper,
   }
}