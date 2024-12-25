use glib::clone;
use gtk::{prelude::*, subclass::text_view};
use std::cell::Cell;
use std::rc::Rc;

use gtk::{glib, Application, ApplicationWindow};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let ui_src = include_str!("waypaper.ui");
    let builder = gtk::Builder::from_string(&ui_src);
    let window = builder
        .object::<gtk::ApplicationWindow>("window")
        .expect("Couldn't get window");
    window.set_application(Some(app));
    let text_view = builder
        .object::<gtk::TextView>("text_view")
        .expect("Couldn't get text_view");
    let text_view_copy = text_view.clone();

    let dialog = gtk::FileDialog::new();

    dialog.select_folder(
        Some(&window),
        gtk::gio::Cancellable::NONE,
        move |o| match o {
            Ok(f) => {
                text_view.buffer().set_text(
                    &String::from_utf8(
                        f.path()
                            .unwrap()
                            .canonicalize()
                            .unwrap()
                            .as_os_str()
                            .as_encoded_bytes()
                            .to_vec(),
                    )
                    .unwrap(),
                );
            }
            Err(_) => todo!(),
        },
    );

    let y = text_view_copy
        .buffer()
        .text(
            &text_view_copy.buffer().start_iter(),
            &text_view_copy.buffer().end_iter(),
            false,
        );
    println!("{}", y.as_str());
    window.present();
}
