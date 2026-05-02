use crate::{
    changers::{hyprpaper::generate_hyprpaper_changer_bar, swaybg::generate_swaybg_changer_bar},
    common::{
        BUTTON_HEIGHT, BUTTON_WIDTH, CacheImageFile, DEFAULT_MARGIN, Wallpaper,
        get_config_file_path, parse_executable_script,
    },
    database::DatabaseConnection,
    wallpaper_changers::{
        HyprpaperFitModes, SwaybgModes, WallpaperChanger, WallpaperChangers,
        get_available_wallpaper_changers,
    },
};
use anyhow::anyhow;
use gettextrs::gettext;
use iced::{
    Alignment::Center,
    Color, Element,
    Length::Fill,
    Subscription, Task,
    application::BootFn,
    event,
    widget::{
        Row, button, column, image, lazy, pick_list, row, scrollable, text, text_input, toggler,
    },
    window,
};
use iced_aw::{
    MenuBar,
    menu::{Item, Menu},
};
use log::{error, trace, warn};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{OpenOptions, remove_file},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use strum::VariantArray;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Serialize, Deserialize, Default, VariantArray, PartialEq)]
pub enum SortBy {
    #[default]
    Date,
    Name,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            SortBy::Date => gettext("Date"),
            SortBy::Name => gettext("Name"),
        };
        write!(f, "{ret}")
    }
}

static WAYLAND_INFO_MONITOR_REGEX: regex_static::once_cell::sync::Lazy<regex::Regex> =
    regex_static::lazy_regex!(r"\(.*\)");

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
    pub monitor: Option<String>,
    sort_by_doc: String,
    pub sort_by: Option<SortBy>,
    invert_sort_doc: String,
    pub invert_sort: bool,
    changer_doc: String,
    pub changer: Option<WallpaperChangers>,
    image_filter_doc: String,
    pub image_filter: String,
    swaybg_mode_doc: String,
    pub swaybg_mode: Option<SwaybgModes>,
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
    #[serde(skip)]
    available_monitors: Vec<String>,
    #[serde(skip)]
    available_changers: Vec<WallpaperChangers>,
    pub hyprpaper_fill_mode: Option<HyprpaperFitModes>,
    #[serde(skip)]
    pub sway_bg_color_internal: Color,
    #[serde(skip)]
    pub show_swaybg_color_picker: bool,
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
            monitor: Default::default(),
            sort_by_doc: gettext(
                "The internal numeric identifier in the changer dropdown used by dconf for the currently selected sorting option. Do not change unless you know what you are doing.",
            ),
            sort_by: Default::default(),
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
            swaybg_mode: Default::default(),
            swaybg_color_doc: gettext(
                "The hex color for swaybg background fill. Must be six characters long.",
            ),
            swaybg_color: Default::default(),
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
            available_monitors: Default::default(),
            available_changers: Default::default(),
            hyprpaper_fill_mode: Default::default(),
            sway_bg_color_internal: Default::default(),
            show_swaybg_color_picker: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum Messages {
    PopulateImageGrid,
    ImageGridPopulated(Vec<CacheImageFile>),
    ChangeWallpaper(PathBuf),
    WallpaperChanged(PathBuf),
    ChangeWallpaperFolder,
    PopulateMonitorDropdown,
    MonitorDropdownPopulated(Vec<String>),
    WallpaperFolderChanged(PathBuf),
    MonitorChanged(String),
    SortByChanged(SortBy),
    SearchBarInputted(String),
    ImagesFiltered(AppStateImages),
    WallpaperChangerChanged(WallpaperChangers),
    InvertSortChanged(bool),
    OptionMenuOpened,
    CloseRequested,
    ExternalScriptExecuted,
    HyprpaperFitModeChanged(HyprpaperFitModes),
    SwaybgModeChanged(SwaybgModes),
    ShowSwaybgColorPicker,
    SwaybgFillColorSubmitted(Color),
    SwaybgFillColorCancelled,
}

impl BootFn<AppState, Messages> for AppState {
    fn boot(&self) -> (AppState, iced::Task<Messages>) {
        let mut instance = self.clone();
        if let None = instance.sort_by {
            instance.sort_by = Some(SortBy::default());
        }
        instance.available_changers = get_available_wallpaper_changers();
        if let None = instance.changer {
            match instance.available_changers.first() {
                Some(c) => {
                    instance.changer = Some(c.clone());
                }
                None => {}
            }
        }
        if let None = instance.hyprpaper_fill_mode {
            match HyprpaperFitModes::VARIANTS.first() {
                Some(h) => {
                    instance.hyprpaper_fill_mode = Some(h.clone());
                }
                None => {}
            }
        }
        if let None = instance.swaybg_mode {
            match SwaybgModes::VARIANTS.first() {
                Some(s) => {
                    instance.swaybg_mode = Some(s.clone());
                }
                None => {}
            }
        }
        (instance, Task::done(Messages::PopulateImageGrid))
    }
}

#[derive(Clone)]
pub struct AppStateImages {
    pub supported_images: Vec<CacheImageFile>,
    pub unsupported_images: Vec<CacheImageFile>,
}

impl AppState {
    pub fn get_config_file() -> anyhow::Result<AppState> {
        let config_file = get_config_file_path()?;
        let mut config = match config_file.exists() {
            true => OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(&config_file)?,
            false => {
                warn!("Config file was not found: Attempting to create a new one.");
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&config_file)?
            }
        };
        let mut config_contents = String::new();
        let _ = config.read_to_string(&mut config_contents)?;

        let config_file_struct = match serde_json::from_str::<AppState>(&config_contents) {
            Ok(s) => {
                trace!("{}", "Successfully obtained configuration file");
                s
            }
            Err(_) => {
                remove_file(&config_file)?;
                config = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&config_file)?;
                let config_file = AppState::default();
                let config_string = serde_json::to_string_pretty::<AppState>(&config_file)?;
                config.write_all(config_string.as_bytes())?;
                config_file
            }
        };

        match parse_executable_script(&config_file_struct.executable_script) {
            Ok(_) => {
                trace!("{}", "Successfully parsed executable script");
            }
            Err(e) => {
                error!("Failed to parse executable script: {e}");
                return Err(anyhow!("Failed to parse executable script: {e}"));
            }
        };
        Ok(config_file_struct)
    }

    fn populate_image_grid(&self) -> iced::Task<Messages> {
        let wallpaper_folder = self.wallpaper_folder.clone();
        let invert_sort = self.invert_sort.clone();
        match &self.sort_by {
            Some(sort_by) => match wallpaper_folder {
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
                None => Task::done(Messages::ImageGridPopulated(vec![])),
            },
            None => Task::none(),
        }
    }

    fn catagorize_images(&self, all_images: &[CacheImageFile]) -> AppStateImages {
        let mut supported_images: Vec<CacheImageFile> = vec![];
        let mut unsupported_images: Vec<CacheImageFile> = vec![];

        match &self.changer {
            Some(changer) => {
                for image in all_images {
                    match changer.accepted_formats().contains(
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
            }
            None => {}
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

    fn get_monitors() -> iced::Task<Messages> {
        Task::future(async move {
            match which::which("wayland-info") {
                Ok(p) => Ok(std::process::Command::new(p)
                    .arg("-i")
                    .arg("wl_output")
                    .output()),
                Err(e) => Err(e),
            }
        })
        .then(|o| {
            let output = match o {
                Ok(o) => o,
                Err(_) => return Task::none(),
            };
            match output {
                Ok(output) => match String::from_utf8(output.stdout) {
                    Ok(output) => {
                        let mut monitors = WAYLAND_INFO_MONITOR_REGEX
                            .find_iter(&output)
                            .map(|m| m.as_str().to_string().replace("(", "").replace(")", ""))
                            .collect::<Vec<_>>();
                        monitors.push(gettext("All"));
                        monitors.sort();
                        Task::done(Messages::MonitorDropdownPopulated(monitors))
                    }
                    Err(_) => Task::none(),
                },
                Err(_) => Task::none(),
            }
        })
    }

    fn sort_image_grid(&mut self, sort_by: SortBy) {
        let comparator = match sort_by {
            SortBy::Date => match &self.invert_sort {
                true => |x: &CacheImageFile, y: &CacheImageFile| y.date.cmp(&x.date),
                false => |x: &CacheImageFile, y: &CacheImageFile| x.date.cmp(&y.date),
            },
            SortBy::Name => match &self.invert_sort {
                true => |x: &CacheImageFile, y: &CacheImageFile| y.name.cmp(&x.name),
                false => |x: &CacheImageFile, y: &CacheImageFile| x.name.cmp(&y.name),
            },
        };
        self.image_grid_images.sort_by(|x, y| comparator(x, y));
    }

    fn filter_images(&self, query: String) -> iced::Task<Messages> {
        let mut images = AppStateImages {
            supported_images: self.image_grid_images.clone(),
            unsupported_images: self.filtered_images.clone(),
        };

        let changer = self.changer.clone();

        Task::future(async move {
            let mut all_images = images.supported_images;
            all_images.append(&mut images.unsupported_images);

            let mut unsupported_images = vec![];

            match changer {
                Some(changer) => {
                    unsupported_images.extend(all_images.extract_if(.., |i| {
                        !changer.accepted_formats().contains(
                            &i.path
                                .extension()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default()
                                .to_string(),
                        )
                    }));

                    unsupported_images
                        .extend(all_images.extract_if(.., |i| !i.name.contains(&query)));
                }
                None => {}
            }

            AppStateImages {
                supported_images: all_images,
                unsupported_images,
            }
        })
        .then(|a| Task::done(Messages::ImagesFiltered(a)))
    }

    fn change_wallpaper(&self, path: PathBuf) -> Task<Messages> {
        let changer = match self.changer.clone() {
            Some(c) => c,
            None => return Task::none(),
        };
        let monitor = match self.monitor.clone() {
            Some(m) => m,
            None => return Task::none(),
        };
        Task::future(async move {
            changer.change(path.clone(), monitor);
            path
        })
        .then(|p| Task::done(Messages::WallpaperChanged(p)))
    }

    fn execute_external_script(&self, wallpaper_path: &Path) -> Task<Messages> {
        let external_script_path = self.executable_script.clone();
	let wallpaper_path = wallpaper_path.to_path_buf();
	let internal_state = self.clone();
	let monitor = self.monitor.clone();
        Task::future(async move {
            let external_script_path = match std::fs::canonicalize(external_script_path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to parse external script path {external_script_path}: {e}");
                    return Task::none();
                }
            };
            let serialized_internal_state = match serde_json::to_string_pretty(&internal_state) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to serialize internal state: {e}");
                    return Task::none();
                }
            };
            let monitor = match monitor {
                Some(m) => m,
                None => {
                    error!("Failed to get monitors");
                    return Task::none();
                }
            };
            match Command::new(external_script_path.to_str().unwrap_or_default())
                .arg(monitor)
                .arg(wallpaper_path)
                .arg(serialized_internal_state)
                .spawn()
            {
                Ok(_) => return Task::done(Messages::ExternalScriptExecuted),
                Err(e) => {
                    error!(
                        "Failed to execute external script {}: {e}",
                        external_script_path.to_str().unwrap_or_default()
                    );
                    return Task::none();
                }
            };
        })
        .then(|t| t)
    }

    pub fn update(&mut self, message: Messages) -> Task<Messages> {
        match message {
            Messages::PopulateImageGrid => self.populate_image_grid(),
            Messages::ImageGridPopulated(i) => {
                let images = self.catagorize_images(&i);
                self.image_grid_images = images.supported_images;
                self.filtered_images = images.unsupported_images;
                Task::done(Messages::PopulateMonitorDropdown)
            }
            Messages::ChangeWallpaper(p) => self.change_wallpaper(p),
            Messages::ChangeWallpaperFolder => Self::open_wallpaper_folder_file_dialog(),
            Messages::WallpaperFolderChanged(f) => {
                self.wallpaper_folder = Some(f);
                self.image_grid_images = vec![];
                self.filtered_images = vec![];
                Task::done(Messages::PopulateImageGrid)
            }
            Messages::PopulateMonitorDropdown => Self::get_monitors(),
            Messages::MonitorDropdownPopulated(monitors) => {
                self.available_monitors = monitors;
                self.monitor = self.available_monitors.first().cloned();
                Task::none()
            }
            Messages::MonitorChanged(m) => {
                self.monitor = Some(m);
                Task::none()
            }
            Messages::SortByChanged(sort_by) => {
                self.sort_image_grid(sort_by);
                Task::none()
            }
            Messages::SearchBarInputted(s) => {
                self.image_filter = s.clone();
                self.filter_images(s)
            }
            Messages::ImagesFiltered(app_state_images) => {
                self.image_grid_images = app_state_images.supported_images;
                self.filtered_images = app_state_images.unsupported_images;
                if let Some(s) = &self.sort_by {
                    self.sort_image_grid(s.clone())
                }
                Task::none()
            }
            Messages::WallpaperChangerChanged(wallpaper_changer) => {
                self.changer = Some(wallpaper_changer);
                self.filter_images(self.image_filter.clone())
            }
            Messages::InvertSortChanged(invert_sort) => {
                self.invert_sort = invert_sort;
                self.filter_images(self.image_filter.clone())
            }
            Messages::OptionMenuOpened => Task::none(),
            Messages::WallpaperChanged(wallpaper_path) => {
                self.execute_external_script(&wallpaper_path)
            }
            Messages::CloseRequested => {
                if let Err(e) = self.write_to_config_file() {
                    error!("Failed to write to config file: {e}");
                }
                window::latest().and_then(window::close)
            }
            Messages::HyprpaperFitModeChanged(hyprpaper_fit_modes) => {
                self.hyprpaper_fill_mode = Some(hyprpaper_fit_modes.clone());
                if let Some(changer) = &self.changer {
                    if let WallpaperChangers::Hyprpaper(_) = changer {
                        self.changer = Some(WallpaperChangers::Hyprpaper(hyprpaper_fit_modes));
                    }
                }
                Task::none()
            }
            Messages::SwaybgModeChanged(swaybg_modes) => {
                self.swaybg_mode = Some(swaybg_modes.clone());
                if let Some(changer) = &self.changer {
                    if let WallpaperChangers::Swaybg(_, color_string) = changer {
                        self.changer = Some(WallpaperChangers::Swaybg(
                            swaybg_modes,
                            color_string.clone(),
                        ));
                    }
                }
                Task::none()
            }
            Messages::SwaybgFillColorSubmitted(color) => {
                self.sway_bg_color_internal = color;
                self.swaybg_color = color.to_string()[0..=color.to_string().len() - 3].to_string();
                self.show_swaybg_color_picker = false;
                if let Some(changer) = &self.changer {
                    if let WallpaperChangers::Swaybg(modes, _) = changer {
                        self.changer = Some(WallpaperChangers::Swaybg(
                            modes.clone(),
                            self.swaybg_color.clone(),
                        ));
                    }
                }
                Task::none()
            }
            Messages::ShowSwaybgColorPicker => {
                self.show_swaybg_color_picker = true;
                Task::none()
            }
            Messages::SwaybgFillColorCancelled => {
                self.show_swaybg_color_picker = false;
                Task::none()
            }
            Messages::ExternalScriptExecuted => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Messages> {
        match &self.changer {
            Some(changer) => {
                let mut image_grid: Row<_> = row![].spacing(DEFAULT_MARGIN as f32);
                for cached_image_file in self.image_grid_images.iter() {
                    image_grid = image_grid.push(lazy(
                        cached_image_file,
                        move |i| -> Element<'_, Messages> {
                            let path = i.path.clone();
                            button(image(&i.cached_image_path).content_fit(iced::ContentFit::Cover))
                                .padding(0)
                                .width(BUTTON_WIDTH)
                                .height(BUTTON_HEIGHT)
                                .on_press_with(move || {
                                    Messages::ChangeWallpaper(PathBuf::from(path.clone()))
                                })
                                .into()
                        },
                    ));
                }

                let monitors_dropdown = pick_list(
                    self.available_monitors.as_slice(),
                    self.monitor.clone(),
                    Messages::MonitorChanged,
                );

                let sort_dropdown = pick_list(
                    SortBy::VARIANTS,
                    self.sort_by.clone(),
                    Messages::SortByChanged,
                );

                let search_bar = text_input(&gettext("Find images"), &self.image_filter)
                    .on_input(Messages::SearchBarInputted)
                    .width(Fill);

                let options_menu: Element<'_, Messages> = MenuBar::new(vec![Item::with_menu(
                    button(text!["{}", gettext("Options")]).on_press(Messages::OptionMenuOpened),
                    Menu::new(
                        [Item::new(
                            toggler(self.invert_sort)
                                .label(gettext("Invert Sort"))
                                .on_toggle(Messages::InvertSortChanged),
                        )]
                        .into(),
                    )
                    .max_width(200.0),
                )])
                .into();

                let changer_dropdown = pick_list(
                    self.available_changers.as_slice(),
                    self.changer.clone(),
                    Messages::WallpaperChangerChanged,
                );

                let changer_specific_widgets = match changer {
                    WallpaperChangers::Hyprpaper(_) => generate_hyprpaper_changer_bar(self),
                    WallpaperChangers::Swaybg(_, _) => generate_swaybg_changer_bar(self),
                    WallpaperChangers::MpvPaper(
                        mpv_paper_pause_modes,
                        mpv_paper_slideshow_settings,
                        _,
                    ) => todo!(),
                    WallpaperChangers::Awww(
                        awwwresize_mode,
                        rgb,
                        awwwscalling_filter,
                        awwwtransition_type,
                        _,
                        _,
                        _,
                        _,
                        awwwtransition_position,
                        _,
                        awwwtransition_bezier,
                        awwwtransition_wave,
                    ) => todo!(),
                    WallpaperChangers::GSlapper(
                        gsllapper_scale_mode,
                        gsllapper_pause_mode,
                        _,
                        _,
                    ) => todo!(),
                };

                column![
                    scrollable(image_grid.wrap()).width(Fill).height(Fill),
                    row![
                        monitors_dropdown,
                        button(text!["{}", gettext("Images Folder")])
                            .on_press(Messages::ChangeWallpaperFolder),
                        sort_dropdown,
                        search_bar,
                        options_menu,
                        changer_dropdown,
                        changer_specific_widgets
                    ]
                    .padding(DEFAULT_MARGIN as f32)
                    .spacing(DEFAULT_MARGIN as f32)
                    .align_y(Center)
                ]
                .align_x(Center)
                .padding(DEFAULT_MARGIN as f32)
                .spacing(DEFAULT_MARGIN as f32)
                .into()
            }
            None => todo!(),
        }
    }

    fn subscription(&self) -> Subscription<Messages> {
        Subscription::filter_map(event::listen(), |event| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                Some(Messages::CloseRequested)
            }
            _ => None,
        })
    }

    pub fn run_application(instance: Self) -> iced::Result {
        iced::application(instance, Self::update, Self::view)
            .centered()
            .subscription(Self::subscription)
            .exit_on_close_request(false)
            .run()
    }
}
