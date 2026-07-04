use clap::Parser;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

pub fn parse_executable_script(s: &str) -> anyhow::Result<String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    let path = s.parse::<PathBuf>()?;
    if !path.metadata()?.is_file() {
        return Err(anyhow::anyhow!("Input is not a file"));
    }
    if path.metadata()?.permissions().mode() & 0o111 == 0 {
        return Err(anyhow::anyhow!("File is not executable"));
    }
    Ok(s.to_owned())
}

#[derive(Parser, Clone)]
pub struct Cli {
    #[arg(short, long)]
    /// Restore previously set wallpapers.
    pub restore: bool,
    #[arg(long, default_value_t = 0)]
    /// How many error, warning, info, debug or trace logs will be shown. 0 for error, 1 for warning, 2 for info, 3 for debug, 4 or higher for trace.
    pub log_level: u8,
    #[arg(short, long, default_value_t = false)]
    /// Get the current wallpaper settings in JSON format.
    pub list_current_wallpapers: bool,
    #[arg(short, long, value_parser = parse_executable_script)]
    /// Path to external script.
    pub external_script: Option<String>,
    #[arg(long)]
    /// Set random wallpapers based on last set changer.
    pub random: bool,
    #[arg(short, long)]
    /// Get application version.
    pub version: bool,
    #[arg(short, long)]
    /// Cycle wallaper(s) the next on based on the previously set wallpaper(s) and sort settings on a given monitor. "All" cycles wallpapers on all monitors.
    pub next: Option<String>,
    #[arg(short, long, default_value_t = 0)]
    /// Startup delay to allow monitors to initialize.
    pub startup_delay: u64,
    #[arg(short, long)]
    /// Delete image cache.
    pub delete_cache: bool,
    #[arg(short = 'b', long)]
    /// Hide bottom bar
    pub hide_bottom_bar: Option<bool>,
}
