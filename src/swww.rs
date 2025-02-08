use crate::{
    common::RGB,
    wallpaper_changers::{
        SWWWScallingFilter, SWWWTransitionBezier, SWWWTransitionPosition, U32Enum,
        WallpaperChangers,
    },
};
use gettextrs::gettext;
use gtk::{
    gdk::RGBA,
    gio::Settings,
    glib::{self, clone},
    prelude::*,
    Adjustment, Align, Box, Button, ColorDialog, ColorDialogButton, DropDown, Entry, Label,
    SpinButton, Switch, TextBuffer, Window,
};
use log::debug;
use std::{path::PathBuf, process::Command};

pub fn change_swww_wallpaper(swww_changer: WallpaperChangers, image: PathBuf, monitor: String) {
    if let WallpaperChangers::Swww(
        resize_modes,
        fill_color,
        scalling_filter,
        transition_type,
        transition_step,
        transition_duration,
        transition_fps,
        transition_angle,
        transition_position,
        invert_y,
        transition_bezier,
        transition_wave,
    ) = swww_changer
    {
        debug!("Starting swww daemon");
        Command::new("swww-daemon").spawn().unwrap().wait().unwrap();
        let mut command = Command::new("swww");
        command
            .arg("img")
            .arg("--resize")
            .arg(resize_modes.to_string())
            .arg("--fill-color")
            .arg(fill_color.to_string());
        if monitor != gettext("All") {
            command.arg("--outputs").arg(monitor);
        }
        command
            .arg("--filter")
            .arg(scalling_filter.to_string())
            .arg("--transition-type")
            .arg(transition_type.to_string())
            .arg("--transition-step")
            .arg(transition_step.to_string())
            .arg("--transition-duration")
            .arg(transition_duration.to_string())
            .arg("--transition-fps")
            .arg(transition_fps.to_string())
            .arg("--transition-angle")
            .arg(transition_angle.to_string())
            .arg("--transition-pos")
            .arg(transition_position.to_string());
        if invert_y {
            command.arg("--invert-y");
        }
        command
            .arg("--transition-bezier")
            .arg(transition_bezier.to_string())
            .arg("--transition-wave")
            .arg(transition_wave.to_string())
            .arg(image)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

pub fn generate_swww_changer_bar(changer_specific_options_box: &Box, settings: Settings) {
    let resize_dropdown =
        DropDown::from_strings(&[&gettext("no"), &gettext("crop"), &gettext("fit")]);
    resize_dropdown.set_margin_top(12);
    resize_dropdown.set_margin_start(12);
    resize_dropdown.set_margin_bottom(12);
    resize_dropdown.set_margin_end(12);
    resize_dropdown.set_halign(Align::Start);
    resize_dropdown.set_valign(Align::Center);
    changer_specific_options_box.append(&resize_dropdown);
    settings
        .bind("swww-resize", &resize_dropdown, "selected")
        .build();
    let color_dialog = ColorDialog::builder().with_alpha(false).build();
    let color_picker = ColorDialogButton::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .dialog(&color_dialog)
        .build();
    let rgb_text_buffer = TextBuffer::builder().build();
    color_picker.connect_rgba_notify(clone!(
        #[weak]
        settings,
        move |b| {
            let rgba = b.rgba();
            let serialize_struct = RGB {
                red: rgba.red(),
                green: rgba.green(),
                blue: rgba.blue(),
            }
            .to_string();
            rgb_text_buffer.set_text(&serialize_struct);
            settings
                .bind("swww-fill-color", &rgb_text_buffer, "text")
                .build();
        }
    ));
    let rgb = settings
        .string("swww-fill-color")
        .to_string()
        .parse::<RGB>()
        .unwrap();
    color_picker.set_rgba(
        &RGBA::builder()
            .red(rgb.red)
            .green(rgb.green)
            .blue(rgb.blue)
            .build(),
    );
    changer_specific_options_box.append(&color_picker);
    let advanced_settings_window = Window::builder()
        .title(gettext("SWWW Advanced Image Settings"))
        .hide_on_close(true)
        .build();
    let advanced_settings_button = Button::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .label(gettext("Advanced Settings"))
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    changer_specific_options_box.append(&advanced_settings_button);
    connect_advanced_settings_window_signals(
        &advanced_settings_button,
        advanced_settings_window,
        settings,
    );
}

fn connect_advanced_settings_window_signals(
    advanced_settings_button: &Button,
    advanced_settings_window: Window,
    settings: Settings,
) {
    advanced_settings_button.connect_clicked(move |_| {
        let advanced_settings_window_box = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_top(12)
            .margin_start(12)
            .margin_bottom(12)
            .margin_end(12)
            .hexpand(true)
            .vexpand(true)
            .build();
        advanced_settings_window.present();
        advanced_settings_window.set_child(Some(&advanced_settings_window_box));
        let filter_options_label = create_label("Scalling filter");
        let filter_dropdown = create_filter_dropdown(&settings);
        let transition_type_label = create_label("Transition type");
        let transition_type_dropdown = create_transition_type_dropdown(&settings);
        let filter_and_transition_box = create_category_box();

        filter_and_transition_box.append(&filter_options_label);
        filter_and_transition_box.append(&filter_dropdown);
        filter_and_transition_box.append(&transition_type_label);
        filter_and_transition_box.append(&transition_type_dropdown);
        advanced_settings_window_box.append(&filter_and_transition_box);

        let transition_step_label = create_label("Transition step");

        let transition_step_adjustment =
            Adjustment::new(90.0, 0.0, f64::from(u8::MAX), 1.0, 0.0, 0.0);
        let transition_step_spinbutton = create_spinbutton(&transition_step_adjustment);

        settings
            .bind("swww-transition-step", &transition_step_spinbutton, "value")
            .build();

        let transition_duration_label = create_label("Transition duration");

        let transition_duration_adjustment =
            Adjustment::new(3.0, 0.0, f64::from(u32::MAX), 1.0, 0.0, 0.0);
        let transition_duration_spinbutton = create_spinbutton(&transition_duration_adjustment);
        settings
            .bind(
                "swww-transition-duration",
                &transition_duration_spinbutton,
                "value",
            )
            .build();

        let transition_angle_label = create_label("Transition angle");

        let transition_angle_adjustment = Adjustment::new(45.0, 0.0, 270.0, 1.0, 0.0, 0.0);
        let transition_angle_spinbutton = create_spinbutton(&transition_angle_adjustment);
        settings
            .bind(
                "swww-transition-angle",
                &transition_angle_spinbutton,
                "value",
            )
            .build();

        let transition_step_duration_angle_box = create_category_box();

        transition_step_duration_angle_box.append(&transition_step_label);
        transition_step_duration_angle_box.append(&transition_step_spinbutton);
        transition_step_duration_angle_box.append(&transition_duration_label);
        transition_step_duration_angle_box.append(&transition_duration_spinbutton);
        transition_step_duration_angle_box.append(&transition_angle_label);
        transition_step_duration_angle_box.append(&transition_angle_spinbutton);
        advanced_settings_window_box.append(&transition_step_duration_angle_box);

        let transition_position_label = create_label("Transition position");

        let transition_position_entry = create_transition_position_entry();

        let transition_position_entry_text_buffer = TextBuffer::builder().build();
        settings
            .bind(
                "swww-transition-position",
                &transition_position_entry_text_buffer,
                "text",
            )
            .build();

        transition_position_entry.set_text(
            transition_position_entry_text_buffer
                .text(
                    &transition_position_entry_text_buffer.start_iter(),
                    &transition_position_entry_text_buffer.end_iter(),
                    false,
                )
                .as_ref(),
        );

        transition_position_entry.connect_changed(move |e| {
            let text = e.text().to_string();
            if SWWWTransitionPosition::new(&text).is_ok() {
                transition_position_entry_text_buffer.set_text(&text)
            }
        });

        let invert_y_label = create_label("Invert Y");

        let invert_y_switch = create_switch("Invert y position in transition position flag");

        settings
            .bind("swww-invert-y", &invert_y_switch, "active")
            .build();

        let transition_wave_label = create_label("Transition wave");

        let transition_wave_width_adjustment =
            Adjustment::new(20.0, 0.0, f64::from(u32::MAX), 1.0, 0.0, 0.0);
        let transition_wave_width_spinbutton = create_spinbutton(&transition_wave_width_adjustment);

        settings
            .bind(
                "swww-transition-wave-width",
                &transition_wave_width_spinbutton,
                "value",
            )
            .build();

        let transition_wave_height_adjustment =
            Adjustment::new(20.0, 0.0, f64::from(u32::MAX), 1.0, 0.0, 0.0);
        let transition_wave_height_spinbutton =
            create_spinbutton(&transition_wave_height_adjustment);

        settings
            .bind(
                "swww-transition-wave-height",
                &transition_wave_height_spinbutton,
                "value",
            )
            .build();

        let transition_position_invert_y_wave_box = create_category_box();

        transition_position_invert_y_wave_box.append(&transition_position_label);
        transition_position_invert_y_wave_box.append(&transition_position_entry);
        transition_position_invert_y_wave_box.append(&invert_y_label);
        transition_position_invert_y_wave_box.append(&invert_y_switch);
        transition_position_invert_y_wave_box.append(&transition_wave_label);
        transition_position_invert_y_wave_box.append(&transition_wave_width_spinbutton);
        transition_position_invert_y_wave_box.append(&transition_wave_height_spinbutton);
        advanced_settings_window_box.append(&transition_position_invert_y_wave_box);

        let transition_bezier_label = create_label("Transition bezier");

        let transition_bezier_adjustments =
            Adjustment::new(0.0, f64::MIN, f64::MAX, 0.01, 0.0, 0.0);
        let transition_bezier_p0_spinbutton =
            create_point_spinbutton(&transition_bezier_adjustments);
        settings
            .bind(
                "swww-transition-bezier-p0",
                &transition_bezier_p0_spinbutton,
                "value",
            )
            .build();
        let transition_bezier_p1_spinbutton =
            create_point_spinbutton(&transition_bezier_adjustments);
        settings
            .bind(
                "swww-transition-bezier-p1",
                &transition_bezier_p1_spinbutton,
                "value",
            )
            .build();
        let transition_bezier_p2_spinbutton =
            create_point_spinbutton(&transition_bezier_adjustments);
        settings
            .bind(
                "swww-transition-bezier-p2",
                &transition_bezier_p2_spinbutton,
                "value",
            )
            .build();
        let transition_bezier_p3_spinbutton =
            create_point_spinbutton(&transition_bezier_adjustments);
        settings
            .bind(
                "swww-transition-bezier-p3",
                &transition_bezier_p3_spinbutton,
                "value",
            )
            .build();

        let transition_bezier_fps_box = create_category_box();

        let transition_frames_per_second_label = create_label("Transition FPS");

        let transition_frames_per_second_adjustment =
            Adjustment::new(30.0, 1.0, f64::from(u32::MAX), 1.0, 0.0, 0.0);

        let transition_frames_per_second_spinbutton =
            create_spinbutton(&transition_frames_per_second_adjustment);

        settings
            .bind(
                "swww-transition-fps",
                &transition_frames_per_second_spinbutton,
                "value",
            )
            .build();

        transition_bezier_fps_box.append(&transition_bezier_label);
        transition_bezier_fps_box.append(&transition_bezier_p0_spinbutton);
        transition_bezier_fps_box.append(&transition_bezier_p1_spinbutton);
        transition_bezier_fps_box.append(&transition_bezier_p2_spinbutton);
        transition_bezier_fps_box.append(&transition_bezier_p3_spinbutton);
        transition_bezier_fps_box.append(&transition_frames_per_second_label);
        transition_bezier_fps_box.append(&transition_frames_per_second_spinbutton);
        advanced_settings_window_box.append(&transition_bezier_fps_box);

        let window_hide_button = create_button("Confirm");

        let restore_defaults_button = create_button("Restore Defaults");

        restore_defaults_button.connect_clicked(move |_| {
            filter_dropdown.set_selected(SWWWScallingFilter::default().to_u32());
            transition_step_spinbutton.set_value(90.0);
            transition_duration_spinbutton.set_value(3.0);
            transition_angle_spinbutton.set_value(45.0);
            transition_position_entry.set_text(&SWWWTransitionPosition::default().to_string());
            invert_y_switch.set_state(false);
            transition_wave_width_spinbutton.set_value(200.0);
            transition_wave_height_spinbutton.set_value(200.0);
            transition_bezier_p0_spinbutton.set_value(SWWWTransitionBezier::default().p0);
            transition_bezier_p1_spinbutton.set_value(SWWWTransitionBezier::default().p1);
            transition_bezier_p2_spinbutton.set_value(SWWWTransitionBezier::default().p2);
            transition_bezier_p3_spinbutton.set_value(SWWWTransitionBezier::default().p3);
            transition_frames_per_second_spinbutton.set_value(30.0);
        });

        let window_control_box = create_window_control_box();

        window_hide_button.connect_clicked(clone!(
            #[weak]
            advanced_settings_window,
            move |_| {
                advanced_settings_window.set_visible(false);
            }
        ));
        window_control_box.append(&restore_defaults_button);
        window_control_box.append(&window_hide_button);
        advanced_settings_window_box.append(&window_control_box);
    });
}

fn create_filter_dropdown(settings: &Settings) -> DropDown {
    let filter_dropdown = DropDown::from_strings(&[
        &gettext("nearest"),
        &gettext("bilinear"),
        &gettext("catmullrom"),
        &gettext("mitchell"),
        &gettext("lanczos3"),
    ]);
    filter_dropdown.set_margin_top(12);
    filter_dropdown.set_margin_start(12);
    filter_dropdown.set_margin_bottom(12);
    filter_dropdown.set_margin_end(12);
    filter_dropdown.set_halign(Align::Start);
    filter_dropdown.set_valign(Align::Center);
    settings
        .bind("swww-scaling-filter", &filter_dropdown, "selected")
        .build();

    filter_dropdown
}

fn create_transition_type_dropdown(settings: &Settings) -> DropDown {
    let transition_type_dropdown = DropDown::from_strings(&[
        &gettext("none"),
        &gettext("simple"),
        &gettext("fade"),
        &gettext("left"),
        &gettext("right"),
        &gettext("top"),
        &gettext("bottom"),
        &gettext("wipe"),
        &gettext("wave"),
        &gettext("grow"),
        &gettext("center"),
        &gettext("any"),
        &gettext("outer"),
        &gettext("random"),
    ]);
    transition_type_dropdown.set_margin_top(12);
    transition_type_dropdown.set_margin_start(12);
    transition_type_dropdown.set_margin_bottom(12);
    transition_type_dropdown.set_margin_end(12);
    transition_type_dropdown.set_halign(Align::Start);
    transition_type_dropdown.set_valign(Align::Center);
    settings
        .bind(
            "swww-transition-type",
            &transition_type_dropdown,
            "selected",
        )
        .build();
    transition_type_dropdown
}

fn create_category_box() -> Box {
    Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build()
}

fn create_button(text: &str) -> Button {
    Button::builder()
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .label(gettext(text))
        .halign(Align::End)
        .valign(Align::Center)
        .build()
}

fn create_spinbutton(adjustment: &Adjustment) -> SpinButton {
    SpinButton::builder()
        .adjustment(adjustment)
        .numeric(true)
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .build()
}

fn create_point_spinbutton(adjustment: &Adjustment) -> SpinButton {
    SpinButton::builder()
        .adjustment(adjustment)
        .numeric(true)
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .build()
}

fn create_label(text: &str) -> Label {
    Label::builder()
        .label(gettext(text))
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .build()
}

fn create_window_control_box() -> Box {
    Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::End)
        .valign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build()
}

fn create_switch(text: &str) -> Switch {
    Switch::builder()
        .tooltip_text(gettext(text))
        .has_tooltip(true)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .margin_end(12)
        .halign(Align::Start)
        .valign(Align::Center)
        .build()
}

fn create_transition_position_entry() -> Entry {
    Entry::builder()
                .placeholder_text(gettext("Transition position"))
                .has_tooltip(true)
                .tooltip_text(gettext("Can either be floating point number between 0 and 0.99, integer coordinate like 200,200 or one of the following: center, top, left, right, bottom, top-left, top-right, bottom-left or bottom-right."))
                .margin_top(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_end(12)
                .halign(Align::Start)
                .valign(Align::Center)
                .build()
}
