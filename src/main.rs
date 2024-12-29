use std::{cell::Ref, cmp::Ordering, path::Path};

use gtk::{
    self, gio::{Cancellable, ListStore, Settings}, glib::{self, clone, BoxedAnyObject}, prelude::*, Align, Application, ApplicationWindow, Box, Button, DropDown, FileDialog, GridView, Label, ListItem, ListScrollFlags, Orientation, ScrollInfo, ScrolledWindow, SignalListItemFactory, SingleSelection, StringObject, Switch, Text, TextBuffer
};

use log::{debug, error};
use waytrogen::{
    common::{GtkImageFile, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH},
    database::DatabaseConnection,
    wallpaper_changers::{Hyprpaper, WallpaperChanger},
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
            .build();
        item.set_child(Some(&button));
    }));

    let folder_path_buffer = TextBuffer::builder().build();
    settings
        .bind("wallpaper-folder", &folder_path_buffer, "text")
        .build();
    log::trace!("Wallpaper Folder: {}",folder_path_buffer.text(&folder_path_buffer.start_iter(), &folder_path_buffer.end_iter(), false));

    insert_images_to_list_store(&folder_path_buffer.text(&folder_path_buffer.start_iter(), &folder_path_buffer.end_iter(), false).to_string(), &image_list_store);
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

    let monitors = gtk::gdk::Display::default().unwrap().monitors();
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

    image_signal_list_item_factory.connect_bind(clone!(
        #[weak]
        monitors_dropdown,
        move |_factory, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let button = item.child().and_downcast::<Button>().unwrap();
            let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let image: Ref<GtkImageFile> = entry.borrow();
            let path = image.clone().path;
            button.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
            button.connect_clicked(move |_| {
                let selected_item = monitors_dropdown
                    .selected_item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string()
                    .to_string();
                match Hyprpaper::change(Path::new(&path), &selected_item) {
                    Ok(_) => {}
                    Err(e) => error!("Could not change wallpaper {} {}", &path, e),
                }
            });
            button.set_child(Some(&image.image));
        }
    ));

    let sort_dropdown = DropDown::from_strings(&["Name", "Date"]);
    let invert_sort_switch = Switch::builder().margin_top(12).margin_bottom(12).margin_start(12).margin_end(12).build();
    let invert_sort_switch_label = Text::builder().text("Invert Sort").margin_start(3).margin_top(12).margin_bottom(12).margin_end(12).build();

    sort_dropdown.connect_selected_notify(clone!(
        #[weak]
        invert_sort_switch,
        #[weak]
        image_list_store,
        #[weak]
        image_grid,
        move |d| {
            sort_images(&d, &invert_sort_switch, &image_list_store, &image_grid);
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
            sort_images(&sort_dropdown, &s, &image_list_store, &image_grid);
        }
    ));

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

    folder_path_buffer.connect_changed(clone!(
        #[weak]
        image_list_store,
        move |f| {
            let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
            insert_images_to_list_store(&path, &image_list_store);
    }));


    window.set_child(Some(&application_box));
}

fn sort_images(sort_dropdown: &DropDown, invert_sort_switch: &Switch, image_list_store: &ListStore, image_grid: &GridView) {
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

fn insert_images_to_list_store(path: &str, image_list_store: &ListStore) {

        image_list_store.remove_all();
        let files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .filter_map(|f| DatabaseConnection::check_cache(f.path()).ok())
            .collect::<Vec<_>>();
        files
            .into_iter()
            .for_each(|g| image_list_store.append(&BoxedAnyObject::new(g)));
}