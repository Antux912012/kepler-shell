use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::{Arc, Mutex};

use super::{CompositorBackend, SnapPosition, WindowInfo, WorkspaceInfo};

pub struct LayerShellHelper;

impl LayerShellHelper {
    pub fn init_taskbar<W: IsA<gtk4::Window> + IsA<gtk4::Widget>>(window: &W, height: i32, is_top: bool) {
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("kepler-taskbar"));
        
        let edge = if is_top { Edge::Top } else { Edge::Bottom };
        window.set_anchor(edge, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        
        window.auto_exclusive_zone_enable();
        window.set_height_request(height);
    }

    pub fn init_flyout<W: IsA<gtk4::Window>>(
        window: &W,
        width: i32,
        height: i32,
        margin_bottom: i32,
        anchor_center: bool,
    ) {
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace(Some("kepler-flyout"));

        window.set_anchor(Edge::Bottom, true);
        if !anchor_center {
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Right, 12);
        }
        window.set_margin(Edge::Bottom, margin_bottom);
        window.set_default_size(width, height);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    }

    pub fn init_osd<W: IsA<gtk4::Window>>(window: &W, width: i32, height: i32) {
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace(Some("kepler-osd"));

        window.set_anchor(Edge::Bottom, true);
        window.set_margin(Edge::Bottom, 60);
        window.set_default_size(width, height);
    }
}

#[derive(Clone)]
pub struct WaylandLayerBackend {
    windows: Arc<Mutex<Vec<WindowInfo>>>,
    active_workspace: Arc<Mutex<u32>>,
}

impl WaylandLayerBackend {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(Vec::new())),
            active_workspace: Arc::new(Mutex::new(0)),
        }
    }
}

impl CompositorBackend for WaylandLayerBackend {
    fn name(&self) -> &'static str {
        "wayland-layer"
    }

    fn get_windows(&self) -> Vec<WindowInfo> {
        self.windows.lock().unwrap().clone()
    }

    fn focus_window(&self, id: &str) -> Result<(), String> {
        let mut wins = self.windows.lock().unwrap();
        for w in wins.iter_mut() {
            w.is_active = w.id == id;
            if w.is_active {
                w.is_minimized = false;
            }
        }
        Ok(())
    }

    fn minimize_window(&self, id: &str) -> Result<(), String> {
        let mut wins = self.windows.lock().unwrap();
        if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
            w.is_minimized = true;
            w.is_active = false;
        }
        Ok(())
    }

    fn close_window(&self, id: &str) -> Result<(), String> {
        let mut wins = self.windows.lock().unwrap();
        wins.retain(|w| w.id != id);
        Ok(())
    }

    fn snap_window(&self, _id: &str, _position: SnapPosition) -> Result<(), String> {
        // Wayland layer backend can delegate tiling commands or forward to foreign-toplevel
        Ok(())
    }

    fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        let active = *self.active_workspace.lock().unwrap();
        (0..4)
            .map(|i| WorkspaceInfo {
                index: i,
                is_active: i == active,
            })
            .collect()
    }

    fn switch_workspace(&self, index: u32) -> Result<(), String> {
        *self.active_workspace.lock().unwrap() = index;
        Ok(())
    }
}
