use std::{fs::File, path::{Path, PathBuf}, str::FromStr, time::UNIX_EPOCH, cell::Ref};

use gtk::{
    gdk::Texture,
    gio::{Cancellable, ListStore},
    glib::{self, clone, BoxedAnyObject},
    prelude::*,
    Align, Application, ApplicationWindow, Box, Button, FileDialog, GridView, Image,
    ListItem, Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection, TextBuffer,
};

use gdk_pixbuf::Pixbuf;

const APP_ID: &str = "org.gtk_rs.HelloWorld1";
const THUMBNAIL_HEIGHT: i32 = 200;
const THUMBNAIL_WIDTH: i32 = THUMBNAIL_HEIGHT;

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

struct PixBufBytes {
    data: Vec<u8>,
    colorspace: PixBufBytesColorSpace,
    has_alpha: bool,
    bits_per_sample: i32,
    width: i32 ,
    height: i32,
    rowstride: i32
}

struct GtkImageFile {
    image: Image,
    name: String,
    date: String,
    path: String
}

impl GtkImageFile {
    pub fn new(path: &str) -> anyhow::Result<GtkImageFile> {
        let image = Image::from_paintable(Some(&Texture::for_pixbuf(
            &gdk_pixbuf::Pixbuf::from_file_at_scale(
path,
                THUMBNAIL_WIDTH,
                THUMBNAIL_HEIGHT,
                true,
            )
            ?,
        )));
        let path = PathBuf::from_str(path)?.canonicalize()?;
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let date = File::open(path.clone())?.metadata()?.created()?;
        let date = date.duration_since(UNIX_EPOCH)?.as_secs().to_string();
        let image_file = GtkImageFile { image, name, date, path: path.to_str().unwrap().to_string() };
        Ok(image_file)
    }
}

#[non_exhaustive]
enum PixBufBytesColorSpace {
   Rgb, 
}

impl FromStr for PixBufBytesColorSpace  {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rgb" => Ok(Self::Rgb),
            _ => Err(())
        }
    }
}


fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Waytrogen")
        .build();

    window.present();

    let image_list_store = ListStore::new::<BoxedAnyObject>();
    for _ in 0..500 {
        // button.set_child(Some(&image));
        let image = GtkImageFile::new("test.png").unwrap();
        image_list_store.append(&BoxedAnyObject::new(image));
    }

    let selection = SingleSelection::new(Some(image_list_store.clone()));
    let image_signal_list_item_factory = SignalListItemFactory::new();
    image_signal_list_item_factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let button = Button::builder()
                .vexpand(true)
                .hexpand(true)
                .can_shrink(true)
                .build();
        item.set_child(Some(
            &button,
        ));
    });

    image_signal_list_item_factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let child = item.child().and_downcast::<Button>().unwrap();
        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let image: Ref<GtkImageFile>= entry.borrow();
        child.set_size_request(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
        child.set_child(Some(&image.image));
    });

    let image_grid = GridView::builder()
        .model(&selection)
        .factory(&image_signal_list_item_factory)
        .max_columns(30)
        .min_columns(3)
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

    let folder_path_buffer = TextBuffer::builder().build();

    let dialog = FileDialog::new();

    open_folder_button.connect_clicked(clone!(
        #[weak]
        folder_path_buffer,
        #[strong]
        window,
        #[weak]
        dialog,
        move |_| {
            dialog.select_folder(Some(&window), Cancellable::NONE, move |d| match d {
                Ok(f) => {
                    folder_path_buffer.set_text(
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
                Err(_) => {}
            });
        }
    ));

    folder_path_buffer.connect_changed(clone!(move |f| {
        let path = f.text(&f.start_iter(), &f.end_iter(), false).to_string();
        let files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter_map(|f| check_cache(f.path()).ok())
            .collect::<Vec<_>>();
        files.iter().for_each(|p| {
            image_list_store.append(&Image::from_paintable(Some(&Texture::for_pixbuf(&p))))
        })
    }));

    window.set_child(Some(&application_box));
}

fn check_cache(path: &Path) -> Result<Pixbuf, glib::Error> {
    return Pixbuf::from_file(path);
}
