pub mod mutter_dbus;
pub mod wayland_layer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub wm_class: String,
    pub workspace: u32,
    pub is_active: bool,
    pub is_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub index: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapPosition {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Maximize,
}

impl SnapPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapPosition::Left => "left",
            SnapPosition::Right => "right",
            SnapPosition::TopLeft => "top-left",
            SnapPosition::TopRight => "top-right",
            SnapPosition::BottomLeft => "bottom-left",
            SnapPosition::BottomRight => "bottom-right",
            SnapPosition::Maximize => "maximize",
        }
    }
}

pub trait CompositorBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_windows(&self) -> Vec<WindowInfo>;
    fn focus_window(&self, id: &str) -> Result<(), String>;
    fn minimize_window(&self, id: &str) -> Result<(), String>;
    fn close_window(&self, id: &str) -> Result<(), String>;
    fn snap_window(&self, id: &str, position: SnapPosition) -> Result<(), String>;
    fn get_workspaces(&self) -> Vec<WorkspaceInfo>;
    fn switch_workspace(&self, index: u32) -> Result<(), String>;
}
