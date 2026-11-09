pub mod theme;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub taskbar: TaskbarConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub start_menu: StartMenuConfig,
    #[serde(default)]
    pub control_center: ControlCenterConfig,
    #[serde(default)]
    pub osd: OsdConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            taskbar: TaskbarConfig::default(),
            theme: ThemeConfig::default(),
            start_menu: StartMenuConfig::default(),
            control_center: ControlCenterConfig::default(),
            osd: OsdConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub autostart: bool,
    #[serde(default = "default_backend")]
    pub compositor_backend: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            compositor_backend: "auto".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskbarConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_alignment")]
    pub alignment: String,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default = "default_icon_size")]
    pub icon_size: i32,
    #[serde(default = "default_pinned")]
    pub pinned_apps: Vec<String>,
    #[serde(default = "default_true")]
    pub show_running_apps: bool,
    #[serde(default = "default_true")]
    pub group_windows: bool,
    #[serde(default = "default_clock_format")]
    pub clock_format: String,
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self {
            position: "bottom".to_string(),
            alignment: "center".to_string(),
            height: 48,
            icon_size: 28,
            pinned_apps: default_pinned(),
            show_running_apps: true,
            group_windows: true,
            clock_format: "%H:%M\n%d/%m/%Y".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_palette")]
    pub palette: String,
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default = "default_true")]
    pub acrylic: bool,
    #[serde(default = "default_blur")]
    pub blur_strength: u32,
    #[serde(default = "default_margin")]
    pub flyout_margin: i32,
    #[serde(default = "default_radius")]
    pub border_radius: i32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            palette: "nord".to_string(),
            dark_mode: true,
            acrylic: true,
            blur_strength: 24,
            flyout_margin: 8,
            border_radius: 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMenuConfig {
    #[serde(default = "default_start_w")]
    pub width: i32,
    #[serde(default = "default_start_h")]
    pub height: i32,
    #[serde(default = "default_search_ph")]
    pub search_placeholder: String,
    #[serde(default = "default_true")]
    pub show_recommended: bool,
    #[serde(default = "default_recent_count")]
    pub max_recent_files: usize,
}

impl Default for StartMenuConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 680,
            search_placeholder: "Type here to search...".to_string(),
            show_recommended: true,
            max_recent_files: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCenterConfig {
    #[serde(default = "default_cc_w")]
    pub width: i32,
    #[serde(default = "default_true")]
    pub show_calendar: bool,
    #[serde(default = "default_true")]
    pub show_media_player: bool,
}

impl Default for ControlCenterConfig {
    fn default() -> Self {
        Self {
            width: 380,
            show_calendar: true,
            show_media_player: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdConfig {
    #[serde(default = "default_osd_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_osd_w")]
    pub width: i32,
    #[serde(default = "default_osd_h")]
    pub height: i32,
}

impl Default for OsdConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 1800,
            width: 240,
            height: 40,
        }
    }
}

fn default_true() -> bool { true }
fn default_backend() -> String { "auto".to_string() }
fn default_position() -> String { "bottom".to_string() }
fn default_alignment() -> String { "center".to_string() }
fn default_height() -> i32 { 48 }
fn default_icon_size() -> i32 { 28 }
fn default_clock_format() -> String { "%H:%M\n%d/%m/%Y".to_string() }
fn default_palette() -> String { "nord".to_string() }
fn default_blur() -> u32 { 24 }
fn default_margin() -> i32 { 8 }
fn default_radius() -> i32 { 12 }
fn default_start_w() -> i32 { 640 }
fn default_start_h() -> i32 { 680 }
fn default_search_ph() -> String { "Type here to search...".to_string() }
fn default_recent_count() -> usize { 6 }
fn default_cc_w() -> i32 { 380 }
fn default_osd_timeout() -> u64 { 1800 }
fn default_osd_w() -> i32 { 240 }
fn default_osd_h() -> i32 { 40 }
fn default_pinned() -> Vec<String> {
    vec![
        "org.gnome.Nautilus".to_string(),
        "org.gnome.Console".to_string(),
        "firefox".to_string(),
        "org.gnome.TextEditor".to_string(),
        "org.gnome.Settings".to_string(),
    ]
}

pub fn get_config_path() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("kepler-shell/config.toml")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/kepler-shell/config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

pub fn load_config() -> (Config, PathBuf) {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str(&content) {
                return (cfg, path);
            }
        }
    }

    // Try fallback locations
    let fallback_paths = [
        PathBuf::from("/usr/share/kepler-shell/config.default.toml"),
        PathBuf::from("data/config.default.toml"),
    ];
    for fb in fallback_paths {
        if fb.exists() {
            if let Ok(content) = fs::read_to_string(&fb) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return (cfg, fb);
                }
            }
        }
    }

    (Config::default(), path)
}

pub fn start_config_watcher(
    path: PathBuf,
    sender: async_channel::Sender<Config>,
) -> notify::Result<RecommendedWatcher> {
    let watch_path = path.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() {
                if let Ok(content) = fs::read_to_string(&watch_path) {
                    if let Ok(new_cfg) = toml::from_str::<Config>(&content) {
                        let _ = sender.send_blocking(new_cfg);
                    }
                }
            }
        }
    })?;

    if let Some(parent) = path.parent() {
        if parent.exists() {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }
    }
    Ok(watcher)
}
