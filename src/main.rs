use std::fmt::Error;

use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let file_filter = gtk::FileFilter::new();
    file_filter.add_mime_type("inode/directory");
    let button = gtk::FileDialog::builder().default_filter(&file_filter).build();


    let window = ApplicationWindow::builder().application(app).title("My GTK App").build();
    let cancel = gtk::gio::Cancellable::new();
    let mut x: Result<gtk::gio::File, gtk::glib::Error> = Err(gtk::glib::Error::new(gtk::FileChooserError::__Unknown(-1), "UwU"));
    button.select_folder(    Some(&window), Some(&cancel),  move |o|  {
        x = o.clone();
    });
    window.present();
}