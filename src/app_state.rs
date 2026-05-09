use crate::locale::TRANSLATION;
use crate::{
    common::{
        BUTTON_HEIGHT, BUTTON_WIDTH, CacheImageFile, DEFAULT_MARGIN, Wallpaper,
        get_config_file_path, parse_executable_script,
    },
    database::DatabaseConnection,
    monitors::AvailableMonitors,
    theme::WaytrogenTheme,
    wallpaper_changers::{
        AWWWResizeMode, AWWWScallingFilter, AWWWTransitionBezier, AWWWTransitionPosition,
        AWWWTransitionType, AWWWTransitionWave, AwwwSettings, GSllaperSettings, GSllapperPauseMode,
        GSllapperScaleMode, HyprpaperFitModes, HyprpaperSettings, MpvPaperPauseModes,
        MpvPaperSettings, MpvPaperSlideshowSettings, SwaybgModes, SwaybgSettings, WallpaperChanger,
        WallpaperChangers, get_available_wallpaper_changers,
    },
};
use anyhow::anyhow;
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
            SortBy::Date => TRANSLATION.get_translation("Date"),
            SortBy::Name => TRANSLATION.get_translation("Name"),
        };
        write!(f, "{ret}")
    }
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
    pub mpvpaper_pause_option: Option<MpvPaperPauseModes>,
    mpvpaper_slideshow_enable_doc: String,
    pub mpvpaper_slideshow_enable: bool,
    mpvpaper_slideshow_interval_doc: String,
    pub mpvpaper_slideshow_interval: u32,
    mpvpaper_additional_options_doc: String,
    pub mpvpaper_additional_options: String,
    selected_monitor_item_doc: String,
    pub selected_monitor_item: String,
    awww_resize_doc: String,
    pub awww_resize: Option<AWWWResizeMode>,
    awww_fill_color_doc: String,
    pub awww_fill_color: String,
    awww_scaling_filter_doc: String,
    pub awww_scaling_filter: Option<AWWWScallingFilter>,
    awww_transition_type_doc: String,
    pub awww_transition_type: Option<AWWWTransitionType>,
    awww_transition_step_doc: String,
    pub awww_transition_step: u8,
    awww_transition_duration_doc: String,
    pub awww_transition_duration: u32,
    awww_transition_angle_doc: String,
    pub awww_transition_angle: u16,
    awww_transition_position_doc: String,
    pub awww_transition_position: String,
    awww_invert_y_doc: String,
    pub awww_invert_y: bool,
    awww_transition_wave_width_doc: String,
    pub awww_transition_wave_width: u32,
    awww_transition_wave_height_doc: String,
    pub awww_transition_wave_height: u32,
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
    pub gslapper_scale_mode: Option<GSllapperScaleMode>,
    gslapper_pause_mode_doc: String,
    pub gslapper_pause_mode: Option<GSllapperPauseMode>,
    gslapper_loop_doc: String,
    pub gslapper_loop: bool,
    gslapper_additional_options_doc: String,
    pub gslapper_additional_options: String,
    hide_changer_options_box_doc: String,
    pub hide_changer_options_box: bool,
    theme_doc: String,
    pub theme: WaytrogenTheme,
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
    #[serde(skip)]
    pub awww_fill_color_internal: Color,
    #[serde(skip)]
    pub show_awww_color_picker: bool,
    #[serde(skip)]
    pub internal_theme: Option<iced::Theme>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            executable_script_doc: TRANSLATION.get_translation("executable-script-description"),
            executable_script: String::default(),
            wallpaper_folder_doc: TRANSLATION.get_translation("current-wallpaper-path-description"),
            wallpaper_folder: None,
            saved_wallpapers_doc: TRANSLATION.get_translation("wallpaper-state-description"),
            saved_wallpapers: vec![Wallpaper::default()],
            monitor_doc: TRANSLATION.get_translation("monitor-dropdown-id-description"),
            monitor: Option::default(),
            sort_by_doc: TRANSLATION.get_translation("sort-by-description"),
            sort_by: Option::default(),
            invert_sort_doc: TRANSLATION.get_translation("invert-sort-description"),
            invert_sort: bool::default(),
            changer_doc: TRANSLATION.get_translation("last-used-changer-description"),
            changer: Option::default(),
            image_filter_doc: TRANSLATION.get_translation("the-search-string-for-the-wallpapers"),
            image_filter: String::default(),
            swaybg_mode_doc: TRANSLATION.get_translation("swaybg-mode-description"),
            swaybg_mode: Option::default(),
            swaybg_color_doc: TRANSLATION.get_translation(
                "the-hex-color-for-swaybg-background-fill-must-be-six-characters-long",
            ),
            swaybg_color: String::default(),
            mpvpaper_pause_option_doc: TRANSLATION
                .get_translation("mpvpaper-pause-mode-description"),
            mpvpaper_pause_option: Option::default(),
            mpvpaper_slideshow_enable_doc: TRANSLATION
                .get_translation("mpvpaper-slidehow-enable-description"),
            mpvpaper_slideshow_enable: bool::default(),
            mpvpaper_slideshow_interval_doc: TRANSLATION
                .get_translation("mpvpaper-slideshow-interval-description"),
            mpvpaper_slideshow_interval: Default::default(),
            mpvpaper_additional_options_doc: TRANSLATION
                .get_translation("custom-options-for-mpvpaper-passed-as-command-line-arguments"),
            mpvpaper_additional_options: String::default(),
            selected_monitor_item_doc: TRANSLATION.get_translation("last-used-monitor-description"),
            selected_monitor_item: String::default(),
            awww_resize_doc: TRANSLATION.get_translation("awww-resize-description"),
            awww_resize: Option::default(),
            awww_fill_color_doc: TRANSLATION.get_translation("awww-fill-color-description"),
            awww_fill_color: String::from("#000000"),
            awww_scaling_filter_doc: TRANSLATION.get_translation("awww-scaling-filter-description"),
            awww_scaling_filter: Option::default(),
            awww_transition_type_doc: TRANSLATION
                .get_translation("awww-transition-mode-description"),
            awww_transition_type: Option::default(),
            awww_transition_step_doc: TRANSLATION
                .get_translation("how-fast-the-transition-approaches-the-new-image-used-by-awww"),
            awww_transition_step: 90,
            awww_transition_duration_doc: TRANSLATION.get_translation(
                "how-long-the-transition-takes-to-complete-in-seconds-used-by-awww",
            ),
            awww_transition_duration: 3,
            awww_transition_angle_doc: TRANSLATION
                .get_translation("awww-transition-angle-description"),
            awww_transition_angle: 45,
            awww_transition_position_doc: TRANSLATION
                .get_translation("awww-circle-center-description"),
            awww_transition_position: "center".to_string(),
            awww_invert_y_doc: TRANSLATION.get_translation("awww-invert-y-description"),
            awww_invert_y: bool::default(),
            awww_transition_wave_width_doc: TRANSLATION
                .get_translation("awww-transition-wave-width-description"),
            awww_transition_wave_width: 200,
            awww_transition_wave_height_doc: TRANSLATION
                .get_translation("awww-transition-wave-height-description"),
            awww_transition_wave_height: 200,
            awww_transition_bezier_p0_doc: TRANSLATION
                .get_translation("point-0-for-the-bezier-curve-to-use-for-the-transition"),
            awww_transition_bezier_p0: 0.54,
            awww_transition_bezier_p1_doc: TRANSLATION
                .get_translation("point-1-for-the-bezier-curve-to-use-for-the-transition"),
            awww_transition_bezier_p1: 0.0,
            awww_transition_bezier_p2_doc: TRANSLATION
                .get_translation("point-2-for-the-bezier-curve-to-use-for-the-transition"),
            awww_transition_bezier_p2: 0.34,
            awww_transition_bezier_p3_doc: TRANSLATION
                .get_translation("point-3-for-the-bezier-curve-to-use-for-the-transition"),
            awww_transition_bezier_p3: 0.99,
            awww_transition_fps_doc: TRANSLATION.get_translation("awww-transition-fps-description"),
            awww_transition_fps: 30,
            gslapper_scale_mode_doc: TRANSLATION.get_translation("gslapper-scale-mode-desciption"),
            gslapper_scale_mode: Option::default(),
            gslapper_pause_mode_doc: TRANSLATION.get_translation("gslapper-pause-mode-description"),
            gslapper_pause_mode: Option::default(),
            gslapper_loop_doc: TRANSLATION.get_translation("gslapper-loop-desciption"),
            gslapper_loop: true,
            gslapper_additional_options_doc: TRANSLATION
                .get_translation("gslapper-additional-options-desciption"),
            gslapper_additional_options: String::default(),
            hide_changer_options_box_doc: TRANSLATION.get_translation("hide-bottom-bar"),
            hide_changer_options_box: false,
            image_grid_images: Vec::default(),
            filtered_images: Vec::default(),
            available_monitors: Vec::default(),
            available_changers: Vec::default(),
            hyprpaper_fill_mode: Option::default(),
            sway_bg_color_internal: Color::default(),
            show_swaybg_color_picker: Default::default(),
            awww_fill_color_internal: Color::default(),
            show_awww_color_picker: Default::default(),
            theme_doc: TRANSLATION.get_translation("theme-description"),
            theme: WaytrogenTheme::default(),
            internal_theme: Option::default(),
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
    MpvPaperPauseModeChanged(MpvPaperPauseModes),
    MpvPaperEnableSlideshowChanged(bool),
    MpvPaperSlideshowIntervalChanged(u32),
    MpvPaperAdditionalOptionsChanged(String),
    AwwwResizeModeChanged(AWWWResizeMode),
    ShowAwwwColorPicker,
    AwwwFillColorSubmitted(Color),
    AwwwFillColorCancelled,
    AwwwAdvancedSettingsButtonClicked,
    AwwwScallingFilterChanged(AWWWScallingFilter),
    AwwwTransitionTypeChanged(AWWWTransitionType),
    AwwwTransitionStepChanged(u8),
    AwwwTransitionDurationChanged(u32),
    AwwwTransitionFPSChanged(u32),
    AwwwTransitionAngleChanged(u16),
    AwwwTransitionPositionChanged(AWWWTransitionPosition),
    AwwwInvertYChanged(bool),
    AwwwTransitionBezierP0Changed(f64),
    AwwwTransitionBezierP1Changed(f64),
    AwwwTransitionBezierP2Changed(f64),
    AwwwTransitionBezierP3Changed(f64),
    AwwwTransitionWaveWidthChanged(u32),
    AwwwTransitionWaveHeightChanged(u32),
    AwwwRestoreDefaults,
    GSllaperScaleModeChanged(GSllapperScaleMode),
    GSlapperPauseModeChanged(GSllapperPauseMode),
    GSllaperLoopVideoChanged(bool),
    GSllaperAdditionalOptionsChanged(String),
    ThemeChanged(iced::Theme),
}

impl BootFn<AppState, Messages> for AppState {
    fn boot(&self) -> (AppState, iced::Task<Messages>) {
        let mut instance = self.clone();
        if instance.sort_by.is_none() {
            instance.sort_by = Some(SortBy::default());
        }
        instance.available_changers = get_available_wallpaper_changers();
        instance.changer = if self.changer.is_some() {
            instance.changer
        } else {
            instance.available_changers.first().cloned()
        };
        if instance.hyprpaper_fill_mode.is_none() {
            instance.hyprpaper_fill_mode = Some(HyprpaperFitModes::default());
        }
        if instance.swaybg_mode.is_none() {
            instance.swaybg_mode = Some(SwaybgModes::default());
        }
        if instance.mpvpaper_pause_option.is_none() {
            instance.mpvpaper_pause_option = Some(MpvPaperPauseModes::default());
        }
        if instance.awww_resize.is_none() {
            instance.awww_resize = Some(AWWWResizeMode::default());
        }
        if instance.awww_scaling_filter.is_none() {
            instance.awww_scaling_filter = Some(AWWWScallingFilter::default());
        }
        if instance.awww_transition_type.is_none() {
            instance.awww_transition_type = Some(AWWWTransitionType::default());
        }
        if !instance.swaybg_color.starts_with('#') {
            instance.swaybg_color = "#000000".to_string();
        }
        if instance.awww_fill_color.is_empty() {
            instance.awww_fill_color = "000000ff".to_string();
        }
        if instance.gslapper_scale_mode.is_none() {
            instance.gslapper_scale_mode = Some(GSllapperScaleMode::default());
        }
        if instance.gslapper_pause_mode.is_none() {
            instance.gslapper_pause_mode = Some(GSllapperPauseMode::default());
        }
        if instance.internal_theme.is_none() {
            instance.internal_theme = Some(instance.theme.0.clone());
        }

        if let Ok(m) = AvailableMonitors::get_monitors() {
            instance.available_monitors = m.available_monitors;
            if instance
                .available_monitors
                .contains(&instance.selected_monitor_item)
            {
                instance.monitor = Some(instance.selected_monitor_item.clone());
            }
        }

        let changer = if let Some(changer) = instance.changer.clone() {
            let c = match changer {
                WallpaperChangers::Hyprpaper(_) => {
                    WallpaperChangers::Hyprpaper(HyprpaperSettings {
                        fit_mode: instance.clone().hyprpaper_fill_mode.unwrap_or_default(),
                    })
                }
                WallpaperChangers::Swaybg(_) => WallpaperChangers::Swaybg(SwaybgSettings {
                    mode: instance.clone().swaybg_mode.unwrap_or_default(),
                    fill_color: instance.swaybg_color.clone(),
                }),
                WallpaperChangers::MpvPaper(_) => WallpaperChangers::MpvPaper(MpvPaperSettings {
                    pause_mode: instance.clone().mpvpaper_pause_option.unwrap_or_default(),
                    slideshow_settings: MpvPaperSlideshowSettings {
                        enable: instance.clone().mpvpaper_slideshow_enable,
                        seconds: instance.clone().mpvpaper_slideshow_interval,
                    },
                    additional_options: instance.clone().mpvpaper_additional_options,
                }),
                WallpaperChangers::Awww(_) => WallpaperChangers::Awww(AwwwSettings {
                    resize_mode: instance.clone().awww_resize.unwrap_or_default(),
                    fill_color: instance.clone().awww_fill_color,
                    scalling_filter: instance.clone().awww_scaling_filter.unwrap_or_default(),
                    transition_type: instance.clone().awww_transition_type.unwrap_or_default(),
                    transition_step: instance.clone().awww_transition_step,
                    transition_duration: instance.clone().awww_transition_duration,
                    transition_fps: instance.clone().awww_transition_fps,
                    transition_angle: instance.clone().awww_transition_angle,
                    transition_position: AWWWTransitionPosition {
                        position: instance.clone().awww_transition_position,
                    },
                    invert_y: instance.clone().awww_invert_y,
                    transition_bezier: AWWWTransitionBezier {
                        p0: instance.clone().awww_transition_bezier_p0,
                        p1: instance.clone().awww_transition_bezier_p1,
                        p2: instance.clone().awww_transition_bezier_p2,
                        p3: instance.clone().awww_transition_bezier_p3,
                    },
                    transition_wave: AWWWTransitionWave {
                        width: instance.clone().awww_transition_wave_width,
                        height: instance.clone().awww_transition_wave_height,
                    },
                }),
                WallpaperChangers::GSlapper(_) => WallpaperChangers::GSlapper(GSllaperSettings {
                    scale_mode: instance.clone().gslapper_scale_mode.unwrap_or_default(),
                    pause_mode: instance.clone().gslapper_pause_mode.unwrap_or_default(),
                    loop_video: instance.clone().gslapper_loop,
                    additional_options: instance.clone().gslapper_additional_options,
                }),
            };
            Some(c)
        } else {
            None
        };

        instance.changer = changer;

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
        let mut config = if config_file.exists() {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(&config_file)?
        } else {
            warn!("Config file was not found: Attempting to create a new one.");
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&config_file)?
        };
        let mut config_contents = String::new();
        let _ = config.read_to_string(&mut config_contents)?;

        let config_file_struct = if let Ok(s) = serde_json::from_str::<AppState>(&config_contents) {
            trace!("{}", "Successfully obtained configuration file");
            s
        } else {
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
        };

        match parse_executable_script(&config_file_struct.executable_script) {
            Ok(_) => {
                trace!("{}", "Successfully parsed executable script");
            }
            Err(e) => {
                error!("Failed to parse executable script: {e}");
                return Err(anyhow!("Failed to parse executable script: {e}"));
            }
        }
        Ok(config_file_struct)
    }

    fn populate_image_grid(&self) -> iced::Task<Messages> {
        let wallpaper_folder = self.wallpaper_folder.clone();
        let invert_sort = self.invert_sort;
        match &self.sort_by {
            Some(sort_by) => match wallpaper_folder {
                Some(wf) => {
                    if !wf.is_dir() {
                        return Task::none();
                    }
                    let accepted_formats = WallpaperChangers::all_accepted_formats();
                    let comparator = match sort_by {
                        SortBy::Date => {
                            if invert_sort {
                                move |x: &DirEntry, y: &DirEntry| {
                                    y.metadata()
                                        .unwrap()
                                        .created()
                                        .unwrap()
                                        .cmp(&x.metadata().unwrap().created().unwrap())
                                }
                            } else {
                                move |x: &DirEntry, y: &DirEntry| {
                                    x.metadata()
                                        .unwrap()
                                        .created()
                                        .unwrap()
                                        .cmp(&y.metadata().unwrap().created().unwrap())
                                }
                            }
                        }
                        SortBy::Name => {
                            if self.invert_sort {
                                move |x: &DirEntry, y: &DirEntry| {
                                    y.file_name()
                                        .to_str()
                                        .unwrap_or_default()
                                        .cmp(x.file_name().to_str().unwrap_or_default())
                                }
                            } else {
                                move |x: &DirEntry, y: &DirEntry| {
                                    x.file_name()
                                        .to_str()
                                        .unwrap_or_default()
                                        .cmp(y.file_name().to_str().unwrap_or_default())
                                }
                            }
                        }
                    };
                    Task::future(async move {
                        let images = WalkDir::new(&wf)
                            .sort_by(comparator)
                            .into_iter()
                            .filter_map(std::result::Result::ok)
                            .map(walkdir::DirEntry::into_path)
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

                        images
                            .into_par_iter()
                            .filter_map(|p| DatabaseConnection::check_cache(&p).ok())
                            .collect::<Vec<_>>()
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

        if let Some(changer) = &self.changer {
            for image in all_images {
                if changer.accepted_formats().contains(
                    &image
                        .cached_image_path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_string(),
                ) {
                    supported_images.push(image.clone());
                } else {
                    unsupported_images.push(image.clone());
                }
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

    fn get_monitors() -> iced::Task<Messages> {
        Task::future(async move {
            match AvailableMonitors::get_monitors() {
                Ok(m) => Task::done(Messages::MonitorDropdownPopulated(m.available_monitors)),
                Err(e) => {
                    error!("Failed to get monitors: {e}");
                    Task::none()
                }
            }
        })
        .then(|o| o)
    }

    fn sort_image_grid(&mut self, sort_by: &SortBy) {
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
        self.image_grid_images.sort_by(comparator);
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

            if let Some(changer) = changer {
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

                unsupported_images.extend(all_images.extract_if(.., |i| !i.name.contains(&query)));
            }

            AppStateImages {
                supported_images: all_images,
                unsupported_images,
            }
        })
        .then(|a| Task::done(Messages::ImagesFiltered(a)))
    }

    fn change_wallpaper(&self, path: PathBuf) -> Task<Messages> {
        let Some(changer) = self.changer.clone() else {
            return Task::none();
        };
        let Some(monitor) = self.monitor.clone() else {
            return Task::none();
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
            let Some(monitor) = monitor else {
                error!("Failed to get monitors");
                return Task::none();
            };
            match Command::new(external_script_path.to_str().unwrap_or_default())
                .arg(monitor)
                .arg(wallpaper_path)
                .arg(serialized_internal_state)
                .spawn()
            {
                Ok(_) => Task::done(Messages::ExternalScriptExecuted),
                Err(e) => {
                    error!(
                        "Failed to execute external script {}: {e}",
                        external_script_path.to_str().unwrap_or_default()
                    );
                    Task::none()
                }
            }
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
                self.filter_images(self.image_filter.clone())
            }
            Messages::ChangeWallpaper(p) => self.change_wallpaper(p),
            Messages::ChangeWallpaperFolder => Self::open_wallpaper_folder_file_dialog(),
            Messages::WallpaperFolderChanged(f) => {
                self.wallpaper_folder = Some(f);
                self.image_grid_images = vec![];
                self.filtered_images = vec![];
                Task::done(Messages::PopulateImageGrid)
            }
            Messages::MonitorDropdownPopulated(monitors) => {
                self.available_monitors = monitors;
                self.monitor = self.available_monitors.first().cloned();
                Task::none()
            }
            Messages::MonitorChanged(m) => {
                self.monitor = Some(m.clone());
                Task::none()
            }
            Messages::SortByChanged(sort_by) => {
                self.sort_image_grid(&sort_by);
                Task::none()
            }
            Messages::SearchBarInputted(s) => {
                self.image_filter.clone_from(&s);
                self.filter_images(s)
            }
            Messages::ImagesFiltered(app_state_images) => {
                self.image_grid_images = app_state_images.supported_images;
                self.filtered_images = app_state_images.unsupported_images;
                if let Some(s) = &self.sort_by.clone() {
                    self.sort_image_grid(s);
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
            Messages::OptionMenuOpened
            | Messages::ExternalScriptExecuted
            | Messages::AwwwAdvancedSettingsButtonClicked
            | Messages::PopulateMonitorDropdown => Task::none(),
            Messages::WallpaperChanged(wallpaper_path) => {
                if let Some(changer) = &self.changer
                    && let Some(monitor) = &self.monitor
                {
                    if monitor == &TRANSLATION.get_translation("All") {
                        self.saved_wallpapers = self
                            .saved_wallpapers
                            .extract_if(.., |i| i.monitor != TRANSLATION.get_translation("All"))
                            .collect::<Vec<_>>();
                    } else {
                        self.saved_wallpapers = self
                            .saved_wallpapers
                            .extract_if(.., |i| i.monitor == TRANSLATION.get_translation("All"))
                            .collect::<Vec<_>>();
                    }
                    match self
                        .saved_wallpapers
                        .iter_mut()
                        .find(|w| w.monitor == *monitor)
                    {
                        Some(w) => {
                            w.changer = changer.clone();
                            w.path = wallpaper_path
                                .clone()
                                .to_str()
                                .unwrap_or_default()
                                .to_string();
                        }
                        None => self.saved_wallpapers.push(Wallpaper {
                            monitor: monitor.clone(),
                            path: wallpaper_path
                                .clone()
                                .to_str()
                                .unwrap_or_default()
                                .to_string(),
                            changer: changer.clone(),
                        }),
                    }
                }
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
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Hyprpaper(_) = changer
                {
                    self.changer = Some(WallpaperChangers::Hyprpaper(HyprpaperSettings {
                        fit_mode: hyprpaper_fit_modes,
                    }));
                }
                Task::none()
            }
            Messages::SwaybgModeChanged(swaybg_modes) => {
                self.swaybg_mode = Some(swaybg_modes.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Swaybg(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Swaybg(SwaybgSettings {
                        mode: swaybg_modes,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::SwaybgFillColorSubmitted(color) => {
                self.sway_bg_color_internal = color;
                self.swaybg_color = color.to_string()[0..=color.to_string().len() - 3].to_string();
                self.show_swaybg_color_picker = false;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Swaybg(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Swaybg(SwaybgSettings {
                        fill_color: self.swaybg_color.clone(),
                        ..settings.clone()
                    }));
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
            Messages::MpvPaperPauseModeChanged(mpv_paper_pause_modes) => {
                self.mpvpaper_pause_option = Some(mpv_paper_pause_modes.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::MpvPaper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::MpvPaper(MpvPaperSettings {
                        pause_mode: mpv_paper_pause_modes,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::MpvPaperEnableSlideshowChanged(enable) => {
                self.mpvpaper_slideshow_enable = enable;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::MpvPaper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::MpvPaper(MpvPaperSettings {
                        slideshow_settings: MpvPaperSlideshowSettings {
                            enable,
                            ..settings.slideshow_settings
                        },
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::MpvPaperSlideshowIntervalChanged(interval) => {
                self.mpvpaper_slideshow_interval = interval;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::MpvPaper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::MpvPaper(MpvPaperSettings {
                        slideshow_settings: MpvPaperSlideshowSettings {
                            seconds: interval,
                            ..settings.slideshow_settings
                        },
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::MpvPaperAdditionalOptionsChanged(additional_options) => {
                self.mpvpaper_additional_options
                    .clone_from(&additional_options);
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::MpvPaper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::MpvPaper(MpvPaperSettings {
                        additional_options,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::AwwwResizeModeChanged(awwwresize_mode) => {
                self.awww_resize = Some(awwwresize_mode.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            resize_mode: awwwresize_mode,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::ShowAwwwColorPicker => {
                self.show_awww_color_picker = true;
                Task::none()
            }
            Messages::AwwwFillColorSubmitted(color) => {
                self.awww_fill_color_internal = color;
                self.awww_fill_color =
                    color.to_string()[0..=color.to_string().len() - 3].to_string();
                self.show_awww_color_picker = false;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            fill_color: self.awww_fill_color.clone(),
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwFillColorCancelled => {
                self.show_awww_color_picker = false;
                Task::none()
            }
            Messages::AwwwScallingFilterChanged(awwwscalling_filter) => {
                self.awww_scaling_filter = Some(awwwscalling_filter.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            scalling_filter: awwwscalling_filter,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionTypeChanged(awwwtransition_type) => {
                self.awww_transition_type = Some(awwwtransition_type.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_type: awwwtransition_type,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionStepChanged(t) => {
                self.awww_transition_step = t;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_step: t,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionDurationChanged(t) => {
                self.awww_transition_duration = t;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_duration: t,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionFPSChanged(f) => {
                self.awww_transition_fps = f;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_fps: f,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionAngleChanged(a) => {
                self.awww_transition_angle = a;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_angle: a,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionPositionChanged(awwwtransition_position) => {
                self.awww_transition_position = awwwtransition_position.clone().position;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_position: awwwtransition_position,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwInvertYChanged(c) => {
                self.awww_invert_y = c;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            invert_y: c,
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionBezierP0Changed(p) => {
                self.awww_transition_bezier_p0 = p;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_bezier: AWWWTransitionBezier {
                                p0: p,
                                ..settings.transition_bezier.clone()
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionBezierP1Changed(p) => {
                self.awww_transition_bezier_p1 = p;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_bezier: AWWWTransitionBezier {
                                p1: p,
                                ..settings.transition_bezier.clone()
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionBezierP2Changed(p) => {
                self.awww_transition_bezier_p2 = p;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_bezier: AWWWTransitionBezier {
                                p2: p,
                                ..settings.transition_bezier.clone()
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionBezierP3Changed(p) => {
                self.awww_transition_bezier_p3 = p;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_bezier: AWWWTransitionBezier {
                                p2: p,
                                ..settings.transition_bezier.clone()
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionWaveWidthChanged(w) => {
                self.awww_transition_wave_width = w;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_wave: AWWWTransitionWave {
                                width: w,
                                ..settings.clone().transition_wave
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwTransitionWaveHeightChanged(h) => {
                self.awww_transition_wave_height = h;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::Awww({
                        AwwwSettings {
                            transition_wave: AWWWTransitionWave {
                                height: h,
                                ..settings.clone().transition_wave
                            },
                            ..settings.clone()
                        }
                    }));
                }
                Task::none()
            }
            Messages::AwwwRestoreDefaults => {
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::Awww(_) = changer
                {
                    let default_settings = AwwwSettings::default();
                    self.awww_scaling_filter = Some(default_settings.scalling_filter);
                    self.awww_transition_step = default_settings.transition_step;
                    self.awww_transition_duration = default_settings.transition_duration;
                    self.awww_transition_position = default_settings.transition_position.position;
                    self.awww_transition_angle = default_settings.transition_angle;
                    self.awww_invert_y = default_settings.invert_y;
                    self.awww_transition_wave_width = default_settings.transition_wave.width;
                    self.awww_transition_wave_height = default_settings.transition_wave.height;
                    self.awww_transition_bezier_p0 = default_settings.transition_bezier.p0;
                    self.awww_transition_bezier_p1 = default_settings.transition_bezier.p1;
                    self.awww_transition_bezier_p2 = default_settings.transition_bezier.p2;
                    self.awww_transition_bezier_p3 = default_settings.transition_bezier.p3;
                    self.awww_transition_fps = default_settings.transition_fps;
                    self.changer = Some(WallpaperChangers::Awww(AwwwSettings::default()));
                }
                Task::none()
            }
            Messages::GSllaperScaleModeChanged(gsllapper_scale_mode) => {
                self.gslapper_scale_mode = Some(gsllapper_scale_mode.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::GSlapper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::GSlapper(GSllaperSettings {
                        scale_mode: gsllapper_scale_mode,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::GSlapperPauseModeChanged(gsllapper_pause_mode) => {
                self.gslapper_pause_mode = Some(gsllapper_pause_mode.clone());
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::GSlapper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::GSlapper(GSllaperSettings {
                        pause_mode: gsllapper_pause_mode,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::GSllaperLoopVideoChanged(b) => {
                self.gslapper_loop = b;
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::GSlapper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::GSlapper(GSllaperSettings {
                        loop_video: b,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::GSllaperAdditionalOptionsChanged(additional_options) => {
                self.gslapper_additional_options
                    .clone_from(&additional_options);
                if let Some(changer) = &self.changer
                    && let WallpaperChangers::GSlapper(settings) = changer
                {
                    self.changer = Some(WallpaperChangers::GSlapper(GSllaperSettings {
                        additional_options,
                        ..settings.clone()
                    }));
                }
                Task::none()
            }
            Messages::ThemeChanged(waytrogen_theme) => {
                self.theme = WaytrogenTheme(waytrogen_theme.clone());
                self.internal_theme = Some(waytrogen_theme);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Messages> {
        match &self.changer {
            Some(changer) => {
                let mut image_grid: Row<_> = row![].spacing(DEFAULT_MARGIN);
                for cached_image_file in &self.image_grid_images {
                    image_grid = image_grid.push(lazy(
                        cached_image_file,
                        move |i| -> Element<'_, Messages> {
                            let path = i.path.clone();
                            button(image(&i.cached_image_path).content_fit(iced::ContentFit::Cover))
                                .padding(0)
                                .width(BUTTON_WIDTH)
                                .height(BUTTON_HEIGHT)
                                .on_press_with(move || Messages::ChangeWallpaper(path.clone()))
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

                let search_bar = text_input(
                    &TRANSLATION.get_translation("find-images"),
                    &self.image_filter,
                )
                .on_input(Messages::SearchBarInputted)
                .width(Fill);

                let options_menu: Element<'_, Messages> = MenuBar::new(vec![Item::with_menu(
                    button(text!["{}", TRANSLATION.get_translation("Options")])
                        .on_press(Messages::OptionMenuOpened),
                    Menu::new(
                        [
                            Item::new(
                                toggler(self.invert_sort)
                                    .label(TRANSLATION.get_translation("invert-sort"))
                                    .on_toggle(Messages::InvertSortChanged),
                            ),
                            Item::new(
                                row![
                                    text!["{}", TRANSLATION.get_translation("theme")],
                                    pick_list(
                                        [
                                            iced::Theme::Light,
                                            iced::Theme::Dark,
                                            iced::Theme::Dracula,
                                            iced::Theme::Nord,
                                            iced::Theme::SolarizedLight,
                                            iced::Theme::SolarizedDark,
                                            iced::Theme::GruvboxLight,
                                            iced::Theme::GruvboxDark,
                                            iced::Theme::CatppuccinLatte,
                                            iced::Theme::CatppuccinFrappe,
                                            iced::Theme::CatppuccinMacchiato,
                                            iced::Theme::CatppuccinMocha,
                                            iced::Theme::TokyoNight,
                                            iced::Theme::TokyoNightStorm,
                                            iced::Theme::TokyoNightLight,
                                            iced::Theme::KanagawaWave,
                                            iced::Theme::KanagawaDragon,
                                            iced::Theme::KanagawaLotus,
                                            iced::Theme::Moonfly,
                                            iced::Theme::Nightfly,
                                            iced::Theme::Oxocarbon,
                                            iced::Theme::Ferra,
                                        ],
                                        self.internal_theme.clone(),
                                        Messages::ThemeChanged,
                                    )
                                ]
                                .spacing(DEFAULT_MARGIN)
                                .width(Fill)
                                .align_y(Center),
                            ),
                        ]
                        .into(),
                    )
                    .max_width(300.0)
                    .spacing(DEFAULT_MARGIN)
                    .padding(DEFAULT_MARGIN),
                )])
                .into();

                let changer_dropdown = pick_list(
                    self.available_changers.as_slice(),
                    self.changer.clone(),
                    Messages::WallpaperChangerChanged,
                );

                let mut bottom_bar = row![
                    monitors_dropdown,
                    button(text!["{}", TRANSLATION.get_translation("image-folder")])
                        .on_press(Messages::ChangeWallpaperFolder),
                    sort_dropdown,
                    search_bar,
                    options_menu,
                    changer_dropdown,
                ]
                .padding(DEFAULT_MARGIN)
                .spacing(DEFAULT_MARGIN)
                .align_y(Center);

                for element in changer.ui_elements(self.clone()) {
                    bottom_bar = bottom_bar.push(element);
                }

                let mut app_box = column![scrollable(image_grid.wrap()).width(Fill).height(Fill),]
                    .align_x(Center)
                    .padding(DEFAULT_MARGIN)
                    .spacing(DEFAULT_MARGIN);

                if !self.hide_changer_options_box {
                    app_box = app_box.push(bottom_bar);
                }

                app_box.into()
            }
            None => column![
                row![
                    text!["{}", TRANSLATION.get_translation("no-changer-available")]
                        .align_x(Center)
                        .width(Fill)
                        .height(Fill)
                ]
                .align_y(Center)
                .height(Fill)
                .width(Fill)
            ]
            .align_x(Center)
            .height(Fill)
            .width(Fill)
            .into(),
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

    fn theme(&self) -> iced::Theme {
        if let Some(theme) = &self.internal_theme {
            theme.clone()
        } else {
            iced::Theme::Dark
        }
    }

    pub fn run_application(instance: Self) -> iced::Result {
        iced::application(instance, Self::update, Self::view)
            .centered()
            .subscription(Self::subscription)
            .exit_on_close_request(false)
            .theme(Self::theme)
            .run()
    }
}
