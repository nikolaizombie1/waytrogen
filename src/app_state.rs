use crate::{
    common::{
        BUTTON_HEIGHT, BUTTON_WIDTH, CacheImageFile, DEFAULT_MARGIN, Wallpaper,
        get_config_file_path,
    },
    database::DatabaseConnection,
    wallpaper_changers::{WallpaperChanger, WallpaperChangers},
};
use gettextrs::gettext;
use iced::{
    Element,
    Length::Fill,
    Task,
    application::BootFn,
    widget::{Row, button, column, image, lazy, row, scrollable, text},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum SortBy {
    #[default]
    Date,
    Name,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    executable_script_doc: String,
    pub executable_script: String,
    wallpaper_folder_doc: String,
    pub wallpaper_folder: Option<PathBuf>,
    saved_wallpapers_doc: String,
    pub saved_wallpapers: Vec<Wallpaper>,
    monitor_doc: String,
    pub monitor: u32,
    sort_by_doc: String,
    pub sort_by: SortBy,
    invert_sort_doc: String,
    pub invert_sort: bool,
    changer_doc: String,
    pub changer: WallpaperChangers,
    image_filter_doc: String,
    pub image_filter: String,
    swaybg_mode_doc: String,
    pub swaybg_mode: u32,
    swaybg_color_doc: String,
    pub swaybg_color: String,
    mpvpaper_pause_option_doc: String,
    pub mpvpaper_pause_option: u32,
    mpvpaper_slideshow_enable_doc: String,
    pub mpvpaper_slideshow_enable: bool,
    mpvpaper_slideshow_interval_doc: String,
    pub mpvpaper_slideshow_interval: f64,
    mpvpaper_additional_options_doc: String,
    pub mpvpaper_additional_options: String,
    selected_monitor_item_doc: String,
    pub selected_monitor_item: String,
    awww_resize_doc: String,
    pub awww_resize: u32,
    awww_fill_color_doc: String,
    pub awww_fill_color: String,
    awww_scaling_filter_doc: String,
    pub awww_scaling_filter: u32,
    awww_transition_type_doc: String,
    pub awww_transition_type: u32,
    awww_transition_step_doc: String,
    pub awww_transition_step: f64,
    awww_transition_duration_doc: String,
    pub awww_transition_duration: f64,
    awww_transition_angle_doc: String,
    pub awww_transition_angle: f64,
    awww_transition_position_doc: String,
    pub awww_transition_position: String,
    awww_invert_y_doc: String,
    pub awww_invert_y: bool,
    awww_transition_wave_width_doc: String,
    pub awww_transition_wave_width: f64,
    awww_transition_wave_height_doc: String,
    pub awww_transition_wave_height: f64,
    awww_transition_bezier_p0_doc: String,
    pub awww_transition_bezier_p0: f64,
    awww_transition_bezier_p1_doc: String,
    pub awww_transition_bezier_p1: f64,
    awww_transition_bezier_p2_doc: String,
    pub awww_transition_bezier_p2: f64,
    awww_transition_bezier_p3_doc: String,
    pub awww_transition_bezier_p3: f64,
    awww_transition_fps_doc: String,
    pub awww_transition_fps: u32,
    gslapper_scale_mode_doc: String,
    pub gslapper_scale_mode: u32,
    gslapper_pause_mode_doc: String,
    pub gslapper_pause_mode: u32,
    gslapper_loop_doc: String,
    pub gslapper_loop: bool,
    gslapper_additional_options_doc: String,
    pub gslapper_additional_options: String,
    hide_changer_options_box_doc: String,
    pub hide_changer_options_box: bool,
    #[serde(skip)]
    image_grid_images: Vec<CacheImageFile>,
    #[serde(skip)]
    filtered_images: Vec<CacheImageFile>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            executable_script_doc: gettext(
                "The path to executable script used after a wallpaper is set. The script is sent the monitor identifier, wallpaper path and the serialized saved_wallpapers state.",
            ),
            executable_script: String::default(),
            wallpaper_folder_doc: gettext(
                "The path to the currently selected wallpaper folder. Note: The path cannot have a trailing forward slash.",
            ),
            wallpaper_folder: None,
            saved_wallpapers_doc: gettext(
                "The collection of the currently saved wallpapers with their corresponding monitor, path and changer.",
            ),
            saved_wallpapers: vec![Wallpaper::default()],
            monitor_doc: gettext(
                "The internal numeric identifier in the monitor dropdown used by dconf for the currently selected monitor. Do not change unless you know what you are doing.",
            ),
            monitor: u32::default(),
            sort_by_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected sorting option. Do not change unless you know what you are doing.",
            ),
            sort_by: SortBy::default(),
            invert_sort_doc: gettext(
                "The boolean flag to invert the currently selected sort-by option in the sort dropdown used by dconf.",
            ),
            invert_sort: bool::default(),
            changer_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected changer. Do not change unless you know what you are doing.",
            ),
            changer: Default::default(),
            image_filter_doc: gettext("The search string for the wallpapers."),
            image_filter: String::default(),
            swaybg_mode_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected swaybg mode. Do not change unless you know what you are doing.",
            ),
            swaybg_mode: u32::default(),
            swaybg_color_doc: gettext(
                "The hex color for swaybg background fill. Must be six characters long.",
            ),
            swaybg_color: String::from("000000"),
            mpvpaper_pause_option_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected mpvpaper pause option. Do not change unless you know what you are doing.",
            ),
            mpvpaper_pause_option: u32::default(),
            mpvpaper_slideshow_enable_doc: gettext(
                "The boolean flag to enable/disable slideshows for mpvpaper used by dconf.",
            ),
            mpvpaper_slideshow_enable: bool::default(),
            mpvpaper_slideshow_interval_doc: gettext(
                "The number of seconds of that mpvpaper takes between switching images in slideshow mode. Note: The option must be a positive floating point number.",
            ),
            mpvpaper_slideshow_interval: f64::default(),
            mpvpaper_additional_options_doc: gettext(
                "Custom options for mpvpaper passed as command line arguments.",
            ),
            mpvpaper_additional_options: String::default(),
            selected_monitor_item_doc: gettext(
                "The currently selected monitor as a string. Note: The name must coincide with the monitor numeric identifier.",
            ),
            selected_monitor_item: String::default(),
            awww_resize_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected awww resize option. Do not change unless you know what you are doing.",
            ),
            awww_resize: u32::default(),
            awww_fill_color_doc: gettext(
                "The hex color for awww background fill. Must be six characters long.",
            ),
            awww_fill_color: String::from("000000"),
            awww_scaling_filter_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected awww scaling filter option. Do not change unless you know what you are doing.",
            ),
            awww_scaling_filter: u32::default(),
            awww_transition_type_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected awww transition type option. Do not change unless you know what you are doing.",
            ),
            awww_transition_type: 1,
            awww_transition_step_doc: gettext(
                "How fast the transition approaches the new image used by awww.",
            ),
            awww_transition_step: 90.0,
            awww_transition_duration_doc: gettext(
                "How long the transition takes to complete in seconds used by awww.",
            ),
            awww_transition_duration: 3.0,
            awww_transition_angle_doc: gettext(
                "Used for the 'wipe' and 'wave' transitions used by awww. It controls the angle of the wipe.",
            ),
            awww_transition_angle: 45.0,
            awww_transition_position_doc: gettext(
                "This is only used for the 'grow','outer' transitions used by awww. It controls the center of circle.",
            ),
            awww_transition_position: String::from("center"),
            awww_invert_y_doc: gettext(
                "Inverts the y position sent in 'transition_pos' flag used by awww.",
            ),
            awww_invert_y: bool::default(),
            awww_transition_wave_width_doc: gettext(
                "Currently only used for 'wave' transition to control the width of each wave used by awww.",
            ),
            awww_transition_wave_width: 200.0,
            awww_transition_wave_height_doc: gettext(
                "Currently only used for 'wave' transition to control the height of each wave used by awww.",
            ),
            awww_transition_wave_height: 200.0,
            awww_transition_bezier_p0_doc: gettext(
                "Point 0 for the Bezier curve to use for the transition",
            ),
            awww_transition_bezier_p0: 0.54,
            awww_transition_bezier_p1_doc: gettext(
                "Point 1 for the Bezier curve to use for the transition",
            ),
            awww_transition_bezier_p1: 0.0,
            awww_transition_bezier_p2_doc: gettext(
                "Point 2 for the Bezier curve to use for the transition",
            ),
            awww_transition_bezier_p2: 0.34,
            awww_transition_bezier_p3_doc: gettext(
                "Point 3 for the Bezier curve to use for the transition",
            ),
            awww_transition_bezier_p3: 0.99,
            awww_transition_fps_doc: gettext("Frame rate for the transition effect used by awww."),
            awww_transition_fps: 30,
            gslapper_scale_mode_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected gslapper scale mode. Do not change unless you know what you are doing.",
            ),
            gslapper_scale_mode: 0,
            gslapper_pause_mode_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected gslapper pause mode. Do not change unless you know what you are doing.",
            ),
            gslapper_pause_mode: 0,
            gslapper_loop_doc: gettext(
                "The boolean flag to loop video wallpapers in gslapper used by dconf.",
            ),
            gslapper_loop: true,
            gslapper_additional_options_doc: gettext(
                "Custom options for gslapper passed as command line arguments.",
            ),
            gslapper_additional_options: String::default(),
            hide_changer_options_box_doc: gettext("Hide bottom bar."),
            hide_changer_options_box: false,
            image_grid_images: Default::default(),
            filtered_images: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum Messages {
    PopulateImageGrid,
    ImageGridPopulated(Vec<CacheImageFile>),
    ChangeWallpaper,
    ChangeWallpaperFolder,
    WallpaperFolderChanged(PathBuf),
}

impl BootFn<AppState, Messages> for AppState {
    fn boot(&self) -> (AppState, iced::Task<Messages>) {
        (self.clone(), Task::done(Messages::PopulateImageGrid))
    }
}

#[derive(Clone)]
pub struct AppStateImages {
    pub supported_images: Vec<CacheImageFile>,
    pub unsupported_images: Vec<CacheImageFile>,
}

impl AppState {
    fn populate_image_grid(&self) -> iced::Task<Messages> {
        let wallpaper_folder = self.wallpaper_folder.clone();
        let sort_by = self.sort_by.clone();
        let invert_sort = self.invert_sort.clone();
        match wallpaper_folder {
            Some(wf) => {
                if !wf.is_dir() {
                    return Task::none();
                }
                let accepted_formats = WallpaperChangers::all_accepted_formats();
                let comparator = match sort_by {
                    SortBy::Date => match invert_sort {
                        true => |x: &DirEntry, y: &DirEntry| {
                            y.metadata()
                                .unwrap()
                                .created()
                                .unwrap()
                                .cmp(&x.metadata().unwrap().created().unwrap())
                        },
                        false => |x: &DirEntry, y: &DirEntry| {
                            x.metadata()
                                .unwrap()
                                .created()
                                .unwrap()
                                .cmp(&y.metadata().unwrap().created().unwrap())
                        },
                    },
                    SortBy::Name => match self.invert_sort {
                        true => |x: &DirEntry, y: &DirEntry| {
                            y.file_name()
                                .to_str()
                                .unwrap_or_default()
                                .cmp(&x.file_name().to_str().unwrap_or_default())
                        },
                        false => |x: &DirEntry, y: &DirEntry| {
                            x.file_name()
                                .to_str()
                                .unwrap_or_default()
                                .cmp(&y.file_name().to_str().unwrap_or_default())
                        },
                    },
                };
                Task::future(async move {
                    let images = WalkDir::new(&wf)
                        .sort_by(move |x, y| comparator(x, y))
                        .into_iter()
                        .filter_map(|d| d.ok())
                        .map(|d| d.into_path())
                        .filter(|d| d.extension().is_some())
                        .filter(|p| {
                            accepted_formats.contains(
                                &p.extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let images = images
                        .into_par_iter()
                        .filter_map(|p| DatabaseConnection::check_cache(&p).ok())
                        .collect::<Vec<_>>();
                    images
                })
                .then(|images| Task::done(Messages::ImageGridPopulated(images)))
            }
            None => Task::none(),
        }
    }

    fn catagorize_images(&self, all_images: &[CacheImageFile]) -> AppStateImages {
        let mut supported_images: Vec<CacheImageFile> = vec![];
        let mut unsupported_images: Vec<CacheImageFile> = vec![];
        for image in all_images {
            match self.changer.accepted_formats().contains(
                &image
                    .cached_image_path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            ) {
                true => supported_images.push(image.clone()),
                false => unsupported_images.push(image.clone()),
            }
        }
        AppStateImages {
            supported_images,
            unsupported_images,
        }
    }

    pub fn write_to_config_file(&self) -> anyhow::Result<()> {
        let config_file = get_config_file_path()?;
        let config_contents = serde_json::to_string_pretty(&self)?;
        let mut config_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&config_file)?;
        config_file.write_all(config_contents.as_bytes())?;
        Ok(())
    }

    fn open_wallpaper_folder_file_dialog() -> Task<Messages> {
        Task::future(
            rfd::AsyncFileDialog::new()
                .set_title("Select Wallpaper Folder")
                .pick_folder(),
        )
        .then(|f| match f {
            Some(folder) => Task::done(Messages::WallpaperFolderChanged(
                folder.path().to_path_buf(),
            )),
            None => Task::none(),
        })
    }

    pub fn update(&mut self, message: Messages) -> iced::Task<Messages> {
        match message {
            Messages::PopulateImageGrid => self.populate_image_grid(),
            Messages::ImageGridPopulated(i) => {
                let images = self.catagorize_images(&i);
                self.image_grid_images = images.supported_images;
                self.filtered_images = images.unsupported_images;
                Task::none()
            }
            Messages::ChangeWallpaper => todo!(),
            Messages::ChangeWallpaperFolder => Self::open_wallpaper_folder_file_dialog(),
            Messages::WallpaperFolderChanged(f) => {
                self.wallpaper_folder = Some(f);
                self.image_grid_images = vec![];
                self.filtered_images = vec![];
                Task::done(Messages::PopulateImageGrid)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Messages> {
        let mut image_grid: Row<_> = row![]
            .padding(DEFAULT_MARGIN as f32)
            .spacing(DEFAULT_MARGIN as f32);
        for cached_image_file in self.image_grid_images.iter() {
            image_grid = image_grid.push(lazy(cached_image_file, |i| -> Element<'_, Messages> {
                button(image(&i.cached_image_path).content_fit(iced::ContentFit::Cover))
                    .padding(0)
                    .width(BUTTON_WIDTH)
                    .height(BUTTON_HEIGHT)
                    .on_press(Messages::ChangeWallpaper)
                    .into()
            }));
        }
        column![
            scrollable(image_grid.wrap()).width(Fill).height(Fill),
            row![
                button(text!["{}", gettext("Images Folder")])
                    .on_press(Messages::ChangeWallpaperFolder)
            ].padding(DEFAULT_MARGIN as f32).spacing(DEFAULT_MARGIN as f32)
        ].padding(DEFAULT_MARGIN as f32).spacing(DEFAULT_MARGIN as f32)
        .into()
    }

    pub fn run_application(instance: Self) -> iced::Result {
        iced::application(instance, Self::update, Self::view)
            .centered()
            .run()
    }
}
