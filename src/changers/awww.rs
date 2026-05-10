use crate::locale::TRANSLATION;
use crate::{
    app_state::{AppState, Messages},
    common::DEFAULT_MARGIN,
    wallpaper_changers::{
        AWWWResizeMode, AWWWScallingFilter, AWWWTransitionPosition, AWWWTransitionType,
        WallpaperChangers,
    },
};
use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    widget::{button, pick_list, row, text, text_input, toggler},
};
use iced_aw::{
    MenuBar,
    helpers::color_picker,
    menu::{Item, Menu},
    number_input,
};
use std::{path::PathBuf, process::Command};
use strum::VariantArray;

pub fn change_awww_wallpaper(awww_changer: WallpaperChangers, image: PathBuf, monitor: String) {
    if let WallpaperChangers::Awww(settings) = awww_changer {
        let awww_daemon_pid = Command::new("pidof")
            .arg("awww-daemon")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        if !awww_daemon_pid.success() {
            Command::new("awww-daemon")
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
        }
        let mut command = Command::new("awww");

        let resize_mode = match settings.resize_mode {
            AWWWResizeMode::No => "no",
            AWWWResizeMode::Crop => "crop",
            AWWWResizeMode::Fit => "fit",
            AWWWResizeMode::Stretch => "stretch",
        };

        let scaling_filter = match settings.scalling_filter {
            AWWWScallingFilter::Nearest => "Nearest",
            AWWWScallingFilter::Bilinear => "Bilinear",
            AWWWScallingFilter::CatmullRom => "CattmullRom",
            AWWWScallingFilter::Mitchell => "Mitchell",
            AWWWScallingFilter::Lanczos3 => "Lanczos3",
        };

        let transition_type = match settings.transition_type {
            AWWWTransitionType::None => "none",
            AWWWTransitionType::Simple => "simple",
            AWWWTransitionType::Fade => "fade",
            AWWWTransitionType::Left => "left",
            AWWWTransitionType::Right => "right",
            AWWWTransitionType::Top => "top",
            AWWWTransitionType::Bottom => "bottom",
            AWWWTransitionType::Wipe => "wipe",
            AWWWTransitionType::Wave => "wave",
            AWWWTransitionType::Grow => "grow",
            AWWWTransitionType::Center => "center",
            AWWWTransitionType::Any => "any",
            AWWWTransitionType::Outer => "outer",
            AWWWTransitionType::Random => "random",
        };

        command
            .arg("img")
            .arg("--resize")
            .arg(resize_mode)
            .arg("--fill-color")
            .arg(&settings.fill_color.replace("#", ""));
        if monitor != TRANSLATION.get_translation("All") {
            command.arg("--outputs").arg(monitor);
        }
        command
            .arg("--filter")
            .arg(scaling_filter)
            .arg("--transition-type")
            .arg(transition_type)
            .arg("--transition-step")
            .arg(settings.transition_step.to_string())
            .arg("--transition-duration")
            .arg(settings.transition_duration.to_string())
            .arg("--transition-fps")
            .arg(settings.transition_fps.to_string())
            .arg("--transition-angle")
            .arg(settings.transition_angle.to_string())
            .arg("--transition-pos")
            .arg(settings.transition_position.to_string());
        if settings.invert_y {
            command.arg("--invert-y");
        }
        command
            .arg("--transition-bezier")
            .arg(settings.transition_bezier.to_string())
            .arg("--transition-wave")
            .arg(settings.transition_wave.to_string())
            .arg(image)
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
    }
}

pub fn generate_awww_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let resize_dropdown: Element<'_, Messages> = pick_list(
        AWWWResizeMode::VARIANTS,
        app_state.awww_resize,
        Messages::AwwwResizeModeChanged,
    )
    .into();

    let color_picker_button = button(text!["{}", TRANSLATION.get_translation("fill-color")])
        .on_press(Messages::ShowAwwwColorPicker);
    let color_picker_widget: Element<'_, Messages> = color_picker(
        app_state.show_awww_color_picker,
        app_state.awww_fill_color_internal,
        color_picker_button,
        Messages::AwwwFillColorCancelled,
        Messages::AwwwFillColorSubmitted,
    )
    .into();

    let advanced_settings_menu: Element<'_, Messages> = MenuBar::new(vec![Item::with_menu(
        button(text![
            "{}",
            TRANSLATION.get_translation("awww-advanced-options")
        ])
        .on_press(Messages::AwwwAdvancedSettingsButtonClicked),
        Menu::new(
            [
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-scaling-filter")],
                        pick_list(
                            AWWWScallingFilter::VARIANTS,
                            app_state.awww_scaling_filter.clone(),
                            Messages::AwwwScallingFilterChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill)
                    .align_y(Center),
                ),
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-transition-type")],
                        pick_list(
                            AWWWTransitionType::VARIANTS,
                            app_state.awww_transition_type.clone(),
                            Messages::AwwwTransitionTypeChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-transition-step")],
                        number_input(
                            &app_state.awww_transition_step,
                            0..=u8::MAX,
                            Messages::AwwwTransitionStepChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-duration")
                        ],
                        number_input(
                            &app_state.awww_transition_duration,
                            0..=u32::MAX,
                            Messages::AwwwTransitionDurationChanged
                        )
                    ]
                    .align_y(Center)
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-transition-angle")],
                        number_input(
                            &app_state.awww_transition_angle,
                            0..=270,
                            Messages::AwwwTransitionAngleChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-position")
                        ],
                        text_input("", &app_state.awww_transition_position).on_input(|m| {
                            Messages::AwwwTransitionPositionChanged(AWWWTransitionPosition {
                                position: m,
                            })
                        })
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-invert-y")],
                        toggler(app_state.awww_invert_y).on_toggle(Messages::AwwwInvertYChanged)
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-wave-height")
                        ],
                        number_input(
                            &app_state.awww_transition_wave_height,
                            0..=u32::MAX,
                            Messages::AwwwTransitionWaveHeightChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-wave-width")
                        ],
                        number_input(
                            &app_state.awww_transition_wave_width,
                            0..=u32::MAX,
                            Messages::AwwwTransitionWaveWidthChanged
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-bezier-p0")
                        ],
                        number_input(
                            &app_state.awww_transition_bezier_p0,
                            f64::MIN..=f64::MAX,
                            Messages::AwwwTransitionBezierP0Changed
                        )
                    ]
                    .align_y(Center)
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-bezier-p1")
                        ],
                        number_input(
                            &app_state.awww_transition_bezier_p1,
                            f64::MIN..=f64::MAX,
                            Messages::AwwwTransitionBezierP1Changed
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-bezier-p2")
                        ],
                        number_input(
                            &app_state.awww_transition_bezier_p2,
                            f64::MIN..=f64::MAX,
                            Messages::AwwwTransitionBezierP2Changed
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text![
                            "{}",
                            TRANSLATION.get_translation("awww-transition-bezier-p3")
                        ],
                        number_input(
                            &app_state.awww_transition_bezier_p3,
                            f64::MIN..=f64::MAX,
                            Messages::AwwwTransitionBezierP3Changed
                        )
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    row![
                        text!["{}", TRANSLATION.get_translation("awww-transition-fps")],
                        number_input(
                            &app_state.awww_transition_fps,
                            0..=u32::MAX,
                            Messages::AwwwTransitionFPSChanged
                        ),
                    ]
                    .spacing(DEFAULT_MARGIN)
                    .align_y(Center)
                    .width(Fill),
                ),
                Item::new(
                    button(text![
                        "{}",
                        TRANSLATION.get_translation("awww-restore-defaults")
                    ])
                    .on_press(Messages::AwwwRestoreDefaults),
                ),
            ]
            .into(),
        )
        .max_width(350.0)
        .spacing(DEFAULT_MARGIN)
        .padding(DEFAULT_MARGIN),
    )])
    .into();

    vec![resize_dropdown, color_picker_widget, advanced_settings_menu]
}
