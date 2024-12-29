use std::{cell::Ref, path::Path};

use gtk::{
    self,
    gio::{Cancellable, ListStore, Settings},
    glib::{self, clone, BoxedAnyObject},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, FileDialog, GridView, ListItem,
    Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection, TextBuffer,
};

use waytrogen::{common::{GtkImageFile, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH}, database::DatabaseConnection};
use log::{warn, trace, debug};

const APP_ID: &str = "org.Waytrogen.Waytrogen";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    stderrlog::new().module(module_path!()).verbosity(5).init().unwrap();

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


    let xdg_dirs = xdg::BaseDirectories::with_prefix("waytrogen").unwrap();
    let cache_path = xdg_dirs.place_cache_file("cache.db").unwrap();

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

    image_signal_list_item_factory.connect_bind(clone!(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let child = item.child().and_downcast::<Button>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let image: Ref<GtkImageFile> = entry.borrow();
        child.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
        child.set_child(Some(&image.image));
    }));

    let folder_path_buffer = TextBuffer::builder().build();
    settings
        .bind("wallpaper-folder", &folder_path_buffer, "text")
        .build();

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
            dialog.select_folder(Some(&window), Cancellable::NONE, move |d| if let Ok(f) = d {
                copy.set_text(f.path().unwrap().canonicalize().unwrap().to_str().unwrap());
            });
        }
    ));

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
    application_box.append(&open_folder_button);


    folder_path_buffer.connect_changed(clone!(
        move |f| {
        let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
        let files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .filter_map(|f| check_cache(f.path(), &cache_path).ok())
            .collect::<Vec<_>>();
        files.into_iter().for_each(|g| image_list_store.append(&BoxedAnyObject::new(g)));
    }));

    selection.connect_selection_changed(clone!(move |s, i, _| {
        s.unselect_all();
        s.set_selected(i);
    }));

    window.set_child(Some(&application_box));
}

fn check_cache(path: &Path, cache_path: &Path) -> Result<GtkImageFile, anyhow::Error> {
    let conn = DatabaseConnection::new(cache_path)?;
    match conn.select_image_file(path) {
        Ok(f) => {
            trace!("Cache Hit: {}", f.path);
            Ok(f)},
        Err(e) => {
            trace!("Cache Miss: {} {}", path.to_str().unwrap(), e);
            match GtkImageFile::from_file(path) {
               Ok(g) => {
                    trace!("GTK Picture created succesfully. {}", g.path);
                    conn.insert_image_file(&g)?;
                    debug!("Picture inserted into database. {}", &g.path);
                    Ok(g)
               },
               Err(e) => {
                    warn!("File could not be converted to a GTK Picture: {} {}", path.to_str().unwrap(), e);
                    Err(e)
               }, 
            }
        }
    }
}
