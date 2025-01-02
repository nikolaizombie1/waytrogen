use std::{
    cell::Ref,
    path::PathBuf,
};

use async_channel::{Receiver, Sender};
use gtk::{
    self,
    gdk::{Display, Texture},
    gio::{spawn_blocking, Cancellable, ListStore, Settings},
    glib::{self, clone, spawn_future_local, BoxedAnyObject, Bytes},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, ColorDialog, ColorDialogButton, DropDown,
    FileDialog, GridView, ListItem, ListScrollFlags, Orientation, Picture, ScrolledWindow,
    SignalListItemFactory, SingleSelection, StringObject, Switch, Text, TextBuffer,
};
use log::{debug, trace};
use strum::IntoEnumIterator;
use waytrogen::{
    common::{CacheImageFile, GtkPictureFile, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH},
    database::DatabaseConnection,
    wallpaper_changers::{SwaybgModes, WallpaperChanger, WallpaperChangers},
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
    if path.is_empty() {
        settings
            .set_string(
                "wallpaper-folder",
                glib::home_dir().to_str().unwrap(),
            )
            .unwrap();
    }
    log::trace!("Wallpaper Folder: {}", path);

    let (sender_cache_images, receiver_cache_images): (Sender<CacheImageFile>, Receiver<CacheImageFile>) =
        async_channel::bounded(1);
    let (sender_enable_changer_options_bar, receiver_changer_options_bar): (Sender<bool>, Receiver<bool>) = async_channel::bounded(1);

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
        #[weak]
        settings,
        move |_| {
            WallpaperChangers::killall_changers();
            change_image_button_handlers(
                image_list_store,
                wallpaper_changers_dropdown,
                monitors_dropdown,
                &settings,
            );
        }
    ));

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
            button.connect_clicked(move |_| {
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
                selected_changer.change(PathBuf::from(&path), selected_monitor)
            });
            button.set_tooltip_text(Some(&image.chache_image_file.name));
            button.set_child(Some(&image.picture));
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
        sender_cache_images.clone(),
        selected_item.clone(),
        invert_sort_switch.state(),
	sender_enable_changer_options_bar.clone()
    );

    let changer_specific_options_box = Box::builder()
        .halign(Align::Center)
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
        .halign(Align::Start)
        .valign(Align::Center)
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
                generate_image_files(path.clone(), sender, selected_item, state, sender_enable_changer_options_bar);
            }));
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

    spawn_future_local(clone!(
	#[strong]
	receiver_changer_options_bar,
	#[weak]
	changer_options_box,
	async move {
	   while let Ok(b) = receiver_changer_options_bar.recv().await {
	      changer_options_box.set_sensitive(b); 
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

fn generate_image_files(
    path: String,
    sender_cache_images: Sender<CacheImageFile>,
    sort_dropdown: String,
    invert_sort_switch_state: bool,
    sender_changer_options: Sender<bool>
) {
    spawn_blocking(move || {
	sender_changer_options.send_blocking(false).expect("The channel must be open");
        let mut files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .map(|d| d.path().to_path_buf())
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
            if let Ok(i) = DatabaseConnection::check_cache(&file) { sender_cache_images.send_blocking(i).expect("The channel must be open") }
        }
	    sender_changer_options.send_blocking(true).expect("The channel must be open");
    });

}

fn get_available_wallpaper_changers() -> Vec<WallpaperChangers> {
    let mut available_changers = vec![];
    for changer in WallpaperChangers::iter() {
        if which(changer.to_string().to_lowercase()).is_ok() {
            available_changers.push(changer);
        }
    }
    available_changers
}

fn change_image_button_handlers(
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

fn generate_changer_bar(
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
            dropdown.set_halign(Align::End);
            dropdown.set_valign(Align::Center);
	    dropdown.set_margin_top(12);
	    dropdown.set_margin_start(12);
	    dropdown.set_margin_bottom(12);
	    dropdown.set_margin_end(12);
            changer_specific_options_box.append(&dropdown);
            let color_dialog = ColorDialog::builder().with_alpha(false).build();
            let color_picker = ColorDialogButton::builder()
                .halign(Align::End)
                .valign(Align::Center)
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .dialog(&color_dialog)
                .build();
            changer_specific_options_box.append(&color_picker);
            settings.bind("swaybg-mode", &dropdown, "selected").build();
            trace!("Binding color-picker");
            trace!("{}", color_picker.rgba().to_string());
            settings.bind("swaybg-color", &color_picker, "rgba").build();
        }
    }
}

fn get_selected_changer(
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
