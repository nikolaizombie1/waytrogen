use crate::{
    app_state::{AppState, Messages},
    common::create_tooltip,
    locale::TRANSLATION,
    wallpaper_changers::{
        GSllaperSettings, GSllapperPauseMode, GSllapperScaleMode, WallpaperChangers,
    },
};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler},
};
use iced_aw::number_input;
use log::debug;
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{net::UnixStream, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};
use strum::VariantArray;

const IPC_TIMEOUT: Duration = Duration::from_secs(2);
const PLAYBACK_TIMEOUT: Duration = Duration::from_secs(6);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const PROBE_INTERVAL: Duration = Duration::from_millis(25);

// ponytail: lifecycle changes are infrequent, so one global lock is enough;
// move to per-output locks if real workloads show contention.
static GSLAPPER_LIFECYCLE: Mutex<()> = Mutex::new(());

fn lifecycle_guard() -> anyhow::Result<MutexGuard<'static, ()>> {
    GSLAPPER_LIFECYCLE
        .lock()
        .map_err(|_| anyhow::anyhow!("gSlapper lifecycle lock is poisoned"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GSlapperStatus {
    pub paused: bool,
    pub media_kind: String,
    pub path: PathBuf,
}

impl GSlapperStatus {
    #[must_use]
    pub fn can_pause(&self) -> bool {
        self.media_kind == "video" && !self.paused
    }

    #[must_use]
    pub fn can_resume(&self) -> bool {
        self.media_kind == "video" && self.paused
    }
}

#[derive(Clone, Debug, Default)]
pub struct GSlapperRuntime {
    pub status: Option<GSlapperStatus>,
    pub cache_status: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum GSlapperControl {
    Pause,
    Resume,
    RefreshCache,
    ClearCache,
    ClearUnusedCache,
}

fn canonical_monitor(monitor: &str) -> &str {
    if monitor == TRANSLATION.get_translation("All") {
        "*"
    } else {
        monitor
    }
}

fn stable_output_id(output: &str) -> u64 {
    // ponytail: FNV-1a keeps socket names stable and short; use a cryptographic
    // hash if output-name collisions appear in the field.
    output.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn managed_socket_path_in(root: &Path, monitor: &str) -> PathBuf {
    root.join(format!(
        "gslapper-{:016x}.sock",
        stable_output_id(canonical_monitor(monitor))
    ))
}

fn managed_runtime_dir() -> anyhow::Result<PathBuf> {
    let root = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| anyhow::anyhow!("XDG_RUNTIME_DIR is not set"))?;
    let dir = PathBuf::from(root).join("waytrogen");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn managed_socket_path(monitor: &str) -> anyhow::Result<PathBuf> {
    Ok(managed_socket_path_in(&managed_runtime_dir()?, monitor))
}

fn is_managed_socket(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("gslapper-") && name.ends_with(".sock"))
}

fn overlapping_socket_paths_in(
    runtime_dir: &Path,
    monitor: &str,
    target_socket: &Path,
) -> anyhow::Result<Vec<PathBuf>> {
    if canonical_monitor(monitor) == "*" {
        Ok(fs::read_dir(runtime_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path != target_socket && is_managed_socket(path))
            .collect())
    } else {
        let all_socket = managed_socket_path_in(runtime_dir, "*");
        Ok(all_socket
            .exists()
            .then_some(all_socket)
            .into_iter()
            .collect())
    }
}

fn stop_overlapping_gslappers(
    runtime_dir: &Path,
    monitor: &str,
    target_socket: &Path,
) -> anyhow::Result<()> {
    for socket in overlapping_socket_paths_in(runtime_dir, monitor, target_socket)? {
        stop_gslapper_at(&socket)?;
    }
    Ok(())
}

fn ipc_request_at(socket: &Path, command: &str) -> anyhow::Result<String> {
    ipc_request_at_with_timeout(socket, command, IPC_TIMEOUT)
}

fn ipc_request_at_with_timeout(
    socket: &Path,
    command: &str,
    read_timeout: Duration,
) -> anyhow::Result<String> {
    if command.contains(['\n', '\r']) {
        anyhow::bail!("gSlapper IPC commands cannot contain newlines");
    }

    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(read_timeout))?;
    stream.set_write_timeout(Some(IPC_TIMEOUT))?;
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    let response = response.trim_end_matches(['\r', '\n']);
    if response.is_empty() {
        anyhow::bail!("gSlapper returned an empty IPC response");
    }
    if let Some(error) = response.strip_prefix("ERROR:") {
        anyhow::bail!("{}", error.trim());
    }
    Ok(response.to_owned())
}

fn parse_status(response: &str) -> anyhow::Result<GSlapperStatus> {
    let mut fields = response.trim_end_matches(['\r', '\n']).splitn(4, ' ');
    if fields.next() != Some("STATUS:") {
        anyhow::bail!("invalid gSlapper status response");
    }
    let paused = match fields.next() {
        Some("paused") => true,
        Some("playing") => false,
        _ => anyhow::bail!("invalid gSlapper playback state"),
    };
    let media_kind = fields
        .next()
        .filter(|field| matches!(*field, "image" | "video"))
        .ok_or_else(|| anyhow::anyhow!("invalid gSlapper media type"))?;
    let path = fields
        .next()
        .filter(|field| !field.is_empty())
        .ok_or_else(|| anyhow::anyhow!("gSlapper status did not include a path"))?;
    Ok(GSlapperStatus {
        paused,
        media_kind: media_kind.to_owned(),
        path: PathBuf::from(path),
    })
}

fn ipc_request(monitor: &str, command: &str) -> anyhow::Result<String> {
    ipc_request_at(&managed_socket_path(monitor)?, command)
}

fn ipc_request_with_timeout(
    monitor: &str,
    command: &str,
    read_timeout: Duration,
) -> anyhow::Result<String> {
    ipc_request_at_with_timeout(&managed_socket_path(monitor)?, command, read_timeout)
}

pub fn query_gslapper(monitor: &str) -> anyhow::Result<GSlapperStatus> {
    parse_status(&ipc_request(monitor, "query")?)
}

pub fn pause_gslapper(monitor: &str) -> anyhow::Result<()> {
    ipc_request_with_timeout(monitor, "pause", PLAYBACK_TIMEOUT).map(|_| ())
}

pub fn resume_gslapper(monitor: &str) -> anyhow::Result<()> {
    ipc_request_with_timeout(monitor, "resume", PLAYBACK_TIMEOUT).map(|_| ())
}

pub fn gslapper_cache_stats(monitor: &str) -> anyhow::Result<String> {
    ipc_request(monitor, "cache-stats")
}

pub fn clear_gslapper_cache(monitor: &str, unused_only: bool) -> anyhow::Result<()> {
    let command = if unused_only {
        "unload unused"
    } else {
        "unload all"
    };
    ipc_request(monitor, command).map(|_| ())
}

pub fn load_gslapper_runtime(monitor: &str) -> anyhow::Result<GSlapperRuntime> {
    let socket = managed_socket_path(monitor)?;
    if !socket.exists() {
        return Ok(GSlapperRuntime::default());
    }
    Ok(GSlapperRuntime {
        status: Some(parse_status(&ipc_request_at(&socket, "query")?)?),
        cache_status: Some(ipc_request_at(&socket, "cache-stats")?),
    })
}

pub fn control_gslapper(
    monitor: &str,
    control: GSlapperControl,
) -> anyhow::Result<GSlapperRuntime> {
    match control {
        GSlapperControl::Pause => pause_gslapper(monitor)?,
        GSlapperControl::Resume => resume_gslapper(monitor)?,
        GSlapperControl::RefreshCache => {}
        GSlapperControl::ClearCache => clear_gslapper_cache(monitor, false)?,
        GSlapperControl::ClearUnusedCache => clear_gslapper_cache(monitor, true)?,
    }
    load_gslapper_runtime(monitor)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChangeRecovery {
    Restart,
    ReturnError,
}

fn classify_change_error(error: &str) -> ChangeRecovery {
    if error.contains("cannot update path (use --auto-stop for video changes)") {
        ChangeRecovery::Restart
    } else {
        ChangeRecovery::ReturnError
    }
}

fn media_path_for_ipc(image: &Path) -> anyhow::Result<&str> {
    let path = image
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("gSlapper IPC requires a UTF-8 media path"))?;
    if path
        .as_bytes()
        .iter()
        .any(|byte| matches!(byte, b'\n' | b'\r'))
    {
        anyhow::bail!("gSlapper media paths cannot contain newlines");
    }
    Ok(path)
}

fn gslapper_help_supports_integration(help: &[u8]) -> bool {
    let help = String::from_utf8_lossy(help);
    ["--ipc-socket", "--transition-type", "--cache-size"]
        .iter()
        .all(|option| help.contains(option))
}

pub fn gslapper_is_supported(executable: &Path) -> bool {
    Command::new(executable)
        .arg("--help")
        .output()
        .is_ok_and(|output| {
            output.status.success() && gslapper_help_supports_integration(&output.stdout)
        })
}

fn ensure_gslapper_is_supported() -> anyhow::Result<()> {
    if gslapper_is_supported(Path::new("gslapper")) {
        Ok(())
    } else {
        anyhow::bail!("gSlapper 1.5 or newer is required")
    }
}

fn launch_args(
    settings: &GSllaperSettings,
    socket: &Path,
    monitor: &str,
    image: &Path,
) -> anyhow::Result<Vec<OsString>> {
    settings.validate().map_err(anyhow::Error::msg)?;

    let mut gst_options = vec![settings.scale_mode.as_arg(), "no-audio"];
    if settings.loop_video {
        gst_options.push("loop");
    }
    let mut gst_options = gst_options.join(" ");
    if !settings.additional_options.trim().is_empty() {
        gst_options.push(' ');
        gst_options.push_str(settings.additional_options.trim());
    }

    let mut args = vec![OsString::from("-I"), socket.as_os_str().to_owned()];
    if let Some(pause_arg) = settings.pause_mode.as_arg() {
        args.push(OsString::from(pause_arg));
    }
    args.extend([
        OsString::from("-o"),
        OsString::from(gst_options),
        OsString::from("-r"),
        OsString::from(settings.fps_cap.to_string()),
        OsString::from("--transition-type"),
        OsString::from(if settings.transition_enabled {
            "fade"
        } else {
            "none"
        }),
        OsString::from("--transition-duration"),
        OsString::from(settings.transition_duration.to_string()),
        OsString::from("--cache-size"),
        OsString::from(settings.cache_size_mb.to_string()),
        OsString::from(canonical_monitor(monitor)),
        image.as_os_str().to_owned(),
    ]);
    Ok(args)
}

fn wait_for_socket(socket: &Path, child: &mut Child) -> anyhow::Result<()> {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        if ipc_request_at(socket, "query").is_ok() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            anyhow::bail!("gSlapper exited before its IPC socket became ready: {status}");
        }
        if Instant::now() >= deadline {
            anyhow::bail!("gSlapper IPC socket did not become ready");
        }
        // ponytail: poll at a fixed short interval; calibrate or add compositor
        // readiness signalling if real hardware exceeds the startup deadline.
        std::thread::sleep(PROBE_INTERVAL);
    }
}

fn terminate_failed_start(child: &mut Child) -> std::io::Result<()> {
    if child.try_wait()?.is_none() {
        child.kill()?;
        child.wait()?;
    }
    Ok(())
}

fn spawn_managed_process(executable: &Path, args: &[OsString]) -> std::io::Result<Child> {
    Command::new(executable)
        .args(args)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

fn spawn_gslapper(
    settings: &GSllaperSettings,
    image: &Path,
    monitor: &str,
    socket: &Path,
) -> anyhow::Result<()> {
    ensure_gslapper_is_supported()?;
    let args = launch_args(settings, socket, monitor, image)?;
    debug!("gSlapper: Running command: gslapper {args:?}");
    let mut child = spawn_managed_process(Path::new("gslapper"), &args)?;
    if let Err(start_error) = wait_for_socket(socket, &mut child) {
        if let Err(cleanup_error) = terminate_failed_start(&mut child) {
            anyhow::bail!(
                "gSlapper startup failed: {start_error}; failed to clean up its process: {cleanup_error}"
            );
        }
        return Err(start_error);
    }
    // ponytail: IPC owns the lifecycle after readiness; a process registry is
    // unnecessary unless gSlapper needs non-IPC recovery after startup.
    Ok(())
}

pub fn apply_gslapper_settings(
    current: &GSllaperSettings,
    settings: &GSllaperSettings,
    monitor: &str,
) -> anyhow::Result<GSlapperRuntime> {
    let _guard = lifecycle_guard()?;
    settings.validate().map_err(anyhow::Error::msg)?;
    let socket = managed_socket_path(monitor)?;
    if !socket.exists() {
        return Ok(GSlapperRuntime::default());
    }
    let status = parse_status(&ipc_request_at(&socket, "query")?)?;

    if current.launch_settings_changed(settings) {
        ensure_gslapper_is_supported()?;
        stop_gslapper_at(&socket)?;
        if let Err(start_error) = spawn_gslapper(settings, &status.path, monitor, &socket) {
            if let Err(rollback_error) = spawn_gslapper(current, &status.path, monitor, &socket) {
                anyhow::bail!(
                    "new gSlapper settings failed: {start_error}; restoring the previous settings also failed: {rollback_error}"
                );
            }
            anyhow::bail!(
                "new gSlapper settings failed; restored previous settings: {start_error}"
            );
        }
    } else {
        if current.transition_enabled != settings.transition_enabled {
            ipc_request_at(
                &socket,
                if settings.transition_enabled {
                    "set-transition fade"
                } else {
                    "set-transition none"
                },
            )?;
        }
        if current.transition_duration != settings.transition_duration {
            ipc_request_at(
                &socket,
                &format!("set-transition-duration {}", settings.transition_duration),
            )?;
        }
    }
    load_gslapper_runtime(monitor)
}

fn stop_gslapper_at(socket: &Path) -> anyhow::Result<()> {
    if !socket.exists() {
        return Ok(());
    }
    if let Err(error) = ipc_request_at(socket, "stop") {
        if UnixStream::connect(socket).is_err() {
            fs::remove_file(socket)?;
            return Ok(());
        }
        return Err(error);
    }

    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        if !socket.exists() {
            return Ok(());
        }
        if UnixStream::connect(socket).is_err() {
            fs::remove_file(socket)?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            anyhow::bail!("gSlapper did not release its IPC socket after stop");
        }
        std::thread::sleep(PROBE_INTERVAL);
    }
}

pub fn stop_gslapper(monitor: &str) -> anyhow::Result<()> {
    let _guard = lifecycle_guard()?;
    stop_gslapper_at(&managed_socket_path(monitor)?)
}

pub fn stop_all_managed_gslappers() -> anyhow::Result<()> {
    let _guard = lifecycle_guard()?;
    let runtime_dir = managed_runtime_dir()?;
    let mut first_error = None;
    for entry in fs::read_dir(runtime_dir)? {
        let path = entry?.path();
        if is_managed_socket(&path)
            && let Err(error) = stop_gslapper_at(&path)
            && first_error.is_none()
        {
            first_error = Some(error);
        }
    }
    first_error.map_or(Ok(()), Err)
}

pub fn change_gslapper_wallpaper(
    gslapper_changer: &WallpaperChangers,
    image: &Path,
    monitor: &str,
) -> anyhow::Result<()> {
    let _guard = lifecycle_guard()?;
    let WallpaperChangers::GSlapper(settings) = gslapper_changer else {
        anyhow::bail!("gSlapper backend called with another wallpaper changer");
    };
    let socket = managed_socket_path(monitor)?;
    let runtime_dir = socket
        .parent()
        .ok_or_else(|| anyhow::anyhow!("managed gSlapper socket has no runtime directory"))?;
    stop_overlapping_gslappers(runtime_dir, monitor, &socket)?;
    let media_path = media_path_for_ipc(image)?;

    if !socket.exists() {
        return spawn_gslapper(settings, image, monitor, &socket);
    }

    if let Err(error) = ipc_request_at(&socket, "query") {
        if UnixStream::connect(&socket).is_ok() {
            return Err(error);
        }
        fs::remove_file(&socket)?;
        return spawn_gslapper(settings, image, monitor, &socket);
    }

    match ipc_request_at(&socket, &format!("change {media_path}")) {
        Ok(_) => Ok(()),
        Err(error) if classify_change_error(&error.to_string()) == ChangeRecovery::Restart => {
            stop_gslapper_at(&socket)?;
            spawn_gslapper(settings, image, monitor, &socket)
        }
        Err(error) => Err(error),
    }
}

pub fn generate_gslapper_changer_bar(app_state: AppState) -> Vec<Element<'static, Messages>> {
    let mut elements: Vec<Element<'static, Messages>> = vec![
        create_tooltip(
            button(text![
                "{}",
                TRANSLATION.get_translation("gslapper-settings")
            ])
            .on_press(Messages::OpenGSlapperSettings)
            .into(),
            text![
                "{}",
                TRANSLATION.get_translation("gslapper-settings-tooltip")
            ]
            .into(),
        )
        .into(),
    ];
    if let Some(error) = app_state.gslapper_error {
        elements.push(text(error).style(text::danger).into());
    }
    elements
}

pub fn generate_gslapper_settings_dialog(app_state: &AppState) -> Element<'_, Messages> {
    let Some(settings) = app_state.gslapper_settings_draft.as_ref() else {
        return column![].into();
    };

    let mut playback_controls = row![].spacing(10).align_y(Alignment::Center);
    let mut pause = button(text!["{}", TRANSLATION.get_translation("gslapper-pause")]);
    let mut resume = button(text!["{}", TRANSLATION.get_translation("gslapper-resume")]);
    if let Some(status) = &app_state.gslapper_status {
        if status.can_pause() {
            pause = pause.on_press(Messages::GSlapperControlRequested(GSlapperControl::Pause));
        }
        if status.can_resume() {
            resume = resume.on_press(Messages::GSlapperControlRequested(GSlapperControl::Resume));
        }
    }
    playback_controls = playback_controls.push(pause).push(resume);

    let status: Element<'_, Messages> = if let Some(status) = &app_state.gslapper_status {
        column![
            text![
                "{}: {}",
                if status.paused {
                    TRANSLATION.get_translation("gslapper-paused")
                } else {
                    TRANSLATION.get_translation("gslapper-playing")
                },
                status.media_kind
            ],
            text(status.path.to_string_lossy()),
            playback_controls,
        ]
        .spacing(6)
        .into()
    } else {
        text!["{}", TRANSLATION.get_translation("gslapper-not-running")].into()
    };

    let behavior = column![
        text![
            "{}",
            TRANSLATION.get_translation("gslapper-wallpaper-behavior")
        ]
        .size(18),
        row![
            text!["{}", TRANSLATION.get_translation("gslapper-scale-mode")],
            pick_list(
                GSllapperScaleMode::VARIANTS,
                Some(settings.scale_mode.clone()),
                Messages::GSllaperScaleModeChanged,
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text!["{}", TRANSLATION.get_translation("gslapper-loop-video")],
            toggler(settings.loop_video).on_toggle(Messages::GSllaperLoopVideoChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text![
                "{}",
                TRANSLATION.get_translation("gslapper-hidden-playback")
            ],
            pick_list(
                GSllapperPauseMode::VARIANTS,
                Some(settings.pause_mode.clone()),
                Messages::GSlapperPauseModeChanged,
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(8);

    let transitions = column![
        text!["{}", TRANSLATION.get_translation("gslapper-transitions")].size(18),
        row![
            text![
                "{}",
                TRANSLATION.get_translation("gslapper-enable-transition")
            ],
            toggler(settings.transition_enabled)
                .on_toggle(Messages::GSlapperTransitionEnabledChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text![
                "{}",
                TRANSLATION.get_translation("gslapper-transition-duration")
            ],
            number_input(
                &settings.transition_duration,
                0.1..=5.0,
                Messages::GSlapperTransitionDurationChanged,
            )
            .step(0.1),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(8);

    let mut content = column![
        text!["{}", TRANSLATION.get_translation("gslapper-settings")].size(24),
        text![
            "{}",
            TRANSLATION.get_translation("gslapper-settings-description")
        ],
        text![
            "{}: {}",
            TRANSLATION.get_translation("gslapper-output"),
            app_state.monitor.as_deref().unwrap_or("-")
        ],
        text!["{}", TRANSLATION.get_translation("gslapper-status")].size(18),
        status,
        behavior,
        transitions,
        button(text![
            "{}",
            if app_state.show_gslapper_advanced {
                TRANSLATION.get_translation("gslapper-hide-advanced")
            } else {
                TRANSLATION.get_translation("gslapper-show-advanced")
            }
        ])
        .on_press(Messages::ToggleGSlapperAdvanced),
    ]
    .spacing(12)
    .width(Length::Fill);

    if app_state.show_gslapper_advanced {
        let cache_status = app_state
            .gslapper_cache_status
            .clone()
            .unwrap_or_else(|| TRANSLATION.get_translation("gslapper-cache-unavailable"));
        let mut refresh = button(text![
            "{}",
            TRANSLATION.get_translation("gslapper-cache-refresh")
        ]);
        let mut clear_unused = button(text![
            "{}",
            TRANSLATION.get_translation("gslapper-cache-clear-unused")
        ]);
        let mut clear_all = button(text![
            "{}",
            TRANSLATION.get_translation("gslapper-cache-clear-all")
        ]);
        if app_state.gslapper_status.is_some() {
            refresh = refresh.on_press(Messages::GSlapperControlRequested(
                GSlapperControl::RefreshCache,
            ));
            clear_unused = clear_unused.on_press(Messages::GSlapperControlRequested(
                GSlapperControl::ClearUnusedCache,
            ));
            clear_all = clear_all.on_press(Messages::GSlapperControlRequested(
                GSlapperControl::ClearCache,
            ));
        }
        content = content.push(
            column![
                row![
                    text!["{}", TRANSLATION.get_translation("gslapper-fps-cap")],
                    pick_list(
                        [30_u16, 60, 100],
                        Some(settings.fps_cap),
                        Messages::GSlapperFpsChanged,
                    ),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text!["{}", TRANSLATION.get_translation("gslapper-cache-size")],
                    number_input(
                        &settings.cache_size_mb,
                        0..=i32::MAX as u32,
                        Messages::GSlapperCacheSizeChanged,
                    ),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text![
                    "{}: {}",
                    TRANSLATION.get_translation("gslapper-cache-status"),
                    cache_status
                ],
                row![refresh, clear_unused, clear_all,].spacing(8),
                text!["{}", TRANSLATION.get_translation("gslapper-raw-options")],
                text![
                    "{}",
                    TRANSLATION.get_translation("gslapper-raw-options-help")
                ]
                .size(12),
                text_input(
                    &TRANSLATION.get_translation("gslapper-additional-option"),
                    &settings.additional_options,
                )
                .on_input(Messages::GSllaperAdditionalOptionsChanged),
            ]
            .spacing(8),
        );
    }

    if let Some(error) = &app_state.gslapper_error {
        content = content.push(text(error).style(text::danger));
    }
    content = content.push(
        row![
            button(text!["{}", TRANSLATION.get_translation("gslapper-cancel")])
                .on_press(Messages::CloseGSlapperSettings),
            button(text!["{}", TRANSLATION.get_translation("gslapper-save")])
                .on_press(Messages::SaveGSlapperSettings),
        ]
        .spacing(10),
    );

    container(scrollable(content).height(Length::Fill))
        .padding(20)
        .width(620)
        .max_height(720)
        .style(container::bordered_box)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallpaper_changers::GSllaperSettings;
    use std::{
        fs,
        io::{BufRead, Write},
        os::unix::net::UnixListener,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        sync::mpsc,
        thread,
    };

    static TEST_SOCKET_ID: AtomicU64 = AtomicU64::new(0);

    fn test_listener() -> (PathBuf, UnixListener, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "waytrogen-gslapper-test-{}-{}",
            std::process::id(),
            TEST_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let socket = root.join("ipc.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        (socket, listener, root)
    }

    #[test]
    fn socket_paths_are_owned_and_output_specific() {
        let root = Path::new("/run/user/1000/waytrogen");
        let first = managed_socket_path_in(root, "DP-1");
        let second = managed_socket_path_in(root, "HDMI-A-1");
        assert_ne!(first, second);
        assert_eq!(first.parent(), Some(root));
        assert!(
            first
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".sock")
        );
    }

    #[test]
    fn wallpaper_changes_are_serialized() {
        let guard = lifecycle_guard().unwrap();
        let (sender, receiver) = mpsc::channel();
        let waiter = thread::spawn(move || {
            let result = change_gslapper_wallpaper(
                &WallpaperChangers::Swaybg(Default::default()),
                Path::new("/tmp/not-used"),
                "DP-1",
            );
            sender.send(result.is_err()).unwrap();
        });

        assert!(receiver.recv_timeout(Duration::from_millis(50)).is_err());
        drop(guard);
        assert!(receiver.recv_timeout(Duration::from_secs(1)).unwrap());
        waiter.join().unwrap();
    }

    #[test]
    fn all_and_specific_outputs_select_only_overlapping_sockets() {
        let root = std::env::temp_dir().join(format!(
            "waytrogen-gslapper-overlap-test-{}-{}",
            std::process::id(),
            TEST_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let all = managed_socket_path_in(&root, "*");
        let dp = managed_socket_path_in(&root, "DP-1");
        let hdmi = managed_socket_path_in(&root, "HDMI-A-1");
        fs::File::create(&all).unwrap();
        fs::File::create(&dp).unwrap();
        fs::File::create(&hdmi).unwrap();

        let for_all = overlapping_socket_paths_in(&root, "*", &all).unwrap();
        assert_eq!(for_all.len(), 2);
        assert!(for_all.contains(&dp));
        assert!(for_all.contains(&hdmi));
        assert_eq!(
            overlapping_socket_paths_in(&root, "DP-1", &dp).unwrap(),
            vec![all]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn query_parser_preserves_spaces_in_paths() {
        let status = parse_status("STATUS: paused image /wallpapers/space name.png\n").unwrap();
        assert!(status.paused);
        assert_eq!(status.path, PathBuf::from("/wallpapers/space name.png"));

        let status = parse_status("STATUS: playing image /wallpapers/trailing  \n").unwrap();
        assert_eq!(status.path, PathBuf::from("/wallpapers/trailing  "));
    }

    #[test]
    fn ipc_request_writes_one_command_and_reads_response() {
        let (socket, listener, root) = test_listener();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut command = String::new();
            std::io::BufReader::new(&mut stream)
                .read_line(&mut command)
                .unwrap();
            assert_eq!(command, "query\n");
            stream
                .write_all(b"STATUS: playing image /tmp/a.png\n")
                .unwrap();
        });

        let response = ipc_request_at(&socket, "query").unwrap();
        server.join().unwrap();
        assert!(response.starts_with("STATUS:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ipc_response_does_not_require_connection_eof() {
        let (socket, listener, root) = test_listener();
        let (release_sender, release_receiver) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut command = String::new();
            std::io::BufReader::new(&mut stream)
                .read_line(&mut command)
                .unwrap();
            assert_eq!(command, "query\n");
            stream
                .write_all(b"STATUS: playing image /tmp/a.png\n")
                .unwrap();
            release_receiver
                .recv_timeout(Duration::from_secs(1))
                .unwrap();
        });

        let response = ipc_request_at_with_timeout(&socket, "query", Duration::from_millis(50));
        release_sender.send(()).unwrap();
        server.join().unwrap();
        assert_eq!(response.unwrap(), "STATUS: playing image /tmp/a.png");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ipc_request_returns_protocol_errors() {
        let (socket, listener, root) = test_listener();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut command = String::new();
            std::io::BufReader::new(&mut stream)
                .read_line(&mut command)
                .unwrap();
            stream.write_all(b"ERROR: no pipeline\n").unwrap();
        });

        let error = ipc_request_at(&socket, "query").unwrap_err();
        server.join().unwrap();
        assert!(error.to_string().contains("no pipeline"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_args_include_gslapper_1_5_settings() {
        let settings = GSllaperSettings {
            scale_mode: GSllapperScaleMode::Stretch,
            pause_mode: GSllapperPauseMode::AutoStop,
            fps_cap: 60,
            transition_enabled: true,
            transition_duration: 0.75,
            cache_size_mb: 128,
            ..GSllaperSettings::default()
        };
        let args = launch_args(
            &settings,
            Path::new("/tmp/socket.sock"),
            "DP-1",
            Path::new("/tmp/wallpaper.mp4"),
        )
        .unwrap();
        let args: Vec<String> = args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(args.windows(2).any(|pair| pair == ["-r", "60"]));
        assert!(args.windows(2).any(|pair| pair == ["--cache-size", "128"]));
        assert!(args.iter().any(|arg| arg == "-s"));
        assert!(args.iter().any(|arg| arg == "stretch no-audio loop"));
        assert!(!args.iter().any(|arg| arg == "-f"));
    }

    #[test]
    fn failed_start_terminates_owned_child() {
        let mut child = Command::new("sleep").arg("10").spawn().unwrap();
        terminate_failed_start(&mut child).unwrap();
        assert!(child.try_wait().unwrap().is_some());
    }

    fn process_group_id(pid: u32) -> u32 {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        stat.rsplit_once(") ")
            .unwrap()
            .1
            .split_whitespace()
            .nth(2)
            .unwrap()
            .parse()
            .unwrap()
    }

    #[test]
    fn managed_child_uses_own_process_group() {
        let args = [OsString::from("10")];
        let mut child = spawn_managed_process(Path::new("sleep"), &args).unwrap();
        let child_pid = child.id();
        let child_process_group = process_group_id(child_pid);
        terminate_failed_start(&mut child).unwrap();

        assert_eq!(child_process_group, child_pid);
    }

    #[test]
    fn startup_reports_child_exit_before_socket_timeout() {
        let socket = std::env::temp_dir().join(format!(
            "waytrogen-missing-gslapper-socket-{}-{}",
            std::process::id(),
            TEST_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut child = Command::new("true").spawn().unwrap();

        let error = wait_for_socket(&socket, &mut child).unwrap_err();

        assert!(error.to_string().contains("exited before"));
    }

    #[test]
    fn compatibility_check_requires_gslapper_1_5_features() {
        let current = b"--ipc-socket PATH --transition-type TYPE --cache-size MB";
        let old = b"--ipc-socket PATH --auto-stop";
        assert!(gslapper_help_supports_integration(current));
        assert!(!gslapper_help_supports_integration(old));
    }

    #[test]
    fn video_change_error_selects_one_restart() {
        assert_eq!(
            classify_change_error("cannot update path (use --auto-stop for video changes)"),
            ChangeRecovery::Restart
        );
        assert_eq!(
            classify_change_error("file not accessible"),
            ChangeRecovery::ReturnError
        );
    }

    #[test]
    fn playback_controls_follow_runtime_status() {
        let mut status = GSlapperStatus {
            paused: false,
            media_kind: "video".to_owned(),
            path: PathBuf::from("wallpaper.mp4"),
        };

        assert!(status.can_pause());
        assert!(!status.can_resume());

        status.paused = true;
        assert!(!status.can_pause());
        assert!(status.can_resume());

        status.media_kind = "image".to_owned();
        assert!(!status.can_pause());
        assert!(!status.can_resume());
    }

    #[test]
    fn playback_timeout_exceeds_gslapper_state_wait() {
        assert!(PLAYBACK_TIMEOUT > Duration::from_secs(5));
    }

    #[test]
    fn stop_waits_until_the_socket_is_released() {
        let (socket, listener, root) = test_listener();
        let server_socket = socket.clone();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut command = String::new();
            std::io::BufReader::new(&mut stream)
                .read_line(&mut command)
                .unwrap();
            assert_eq!(command, "stop\n");
            stream.write_all(b"OK\n").unwrap();
            drop(stream);
            drop(listener);
            thread::sleep(Duration::from_millis(50));
            let _ = fs::remove_file(server_socket);
        });

        stop_gslapper_at(&socket).unwrap();
        server.join().unwrap();
        assert!(!socket.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
