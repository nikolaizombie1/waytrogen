use std::sync::OnceLock;

use gtk::{
    gdk::Texture, gio::{self, ListStore}, glib::{self, subclass::Signal, BoxedAnyObject, Value, closure_local, clone}, prelude::*, Align, Application, ApplicationWindow, Box, Button, GridView, Image, ListItem, Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection
};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let image_list_store = ListStore::new::<Image>();
    for _ in 0..100 {
        let image = Image::from_paintable(Some(
            &Texture::from_file(&gio::File::for_path("test.png")).unwrap(),
        ));
        // button.set_child(Some(&image));
        image_list_store.append(&image);
    }

    let selection = SingleSelection::new(Some(image_list_store));
    let image_signal_list_item_factory = SignalListItemFactory::new();
    image_signal_list_item_factory.connect_setup( move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        item.set_child(Some(&Button::builder().vexpand(true).hexpand(true).can_shrink(true).build()));
    });

    image_signal_list_item_factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let child = item.child();
        let child = child.and_downcast::<Button>().unwrap();
        let entry = item.item().and_downcast::<Image>().unwrap();
        child.set_size_request(200, 200);
        child.set_child(Some(&entry));
    });


    let image_grid = GridView::builder().model(&selection).factory(&image_signal_list_item_factory).hexpand(true).vexpand(true).build();
    let scrolled_winow = ScrolledWindow::builder().child(&image_grid).valign(Align::Start).halign(Align::Center).propagate_natural_height(true).propagate_natural_width(true).build();
    scrolled_winow.set_size_request(900, 900);
    let open_folder_button = Button::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::End)
        .build();
    let application_box = Box::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .orientation(Orientation::Vertical)
        .build();
    application_box.append(&scrolled_winow);
    application_box.append(&open_folder_button);

    application_box.set_width_request(1000);

    open_folder_button.set_label("Image Folder");

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Waytrogen")
        .child(&application_box)
        .build();

        let window_copy = window.clone();
    //application_box.connect_closure("size-allocate", true,  c
    

    window.present();
}
