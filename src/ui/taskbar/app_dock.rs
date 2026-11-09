use gio::prelude::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Orientation};
use std::collections::HashSet;
use std::sync::Arc;

use crate::backend::{CompositorBackend, WindowInfo};

pub struct AppDock {
    container: Box,
    backend: Arc<dyn CompositorBackend>,
    pinned_apps: Vec<String>,
}

impl AppDock {
    pub fn new(pinned_apps: &[String], backend: Arc<dyn CompositorBackend>) -> Self {
        let container = Box::new(Orientation::Horizontal, 4);
        container.set_valign(gtk4::Align::Center);
        container.set_halign(gtk4::Align::Center);

        let dock = Self {
            container,
            backend,
            pinned_apps: pinned_apps.to_vec(),
        };

        dock.refresh(&dock.backend.get_windows());
        dock
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn find_app_info(app_id: &str) -> Option<gio::AppInfo> {
        let target = app_id.trim_end_matches(".desktop").to_lowercase();
        gio::AppInfo::all().into_iter().find(|a| {
            if let Some(id) = a.id() {
                let id_clean = id.trim_end_matches(".desktop").to_lowercase();
                if id_clean == target || id_clean.ends_with(&format!(".{}", target)) {
                    return true;
                }
            }
            if a.name().to_lowercase() == target {
                return true;
            }
            false
        })
    }

    pub fn refresh(&self, windows: &[WindowInfo]) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        let mut seen_apps = HashSet::new();

        // 1. Pinned Apps
        for app_id in &self.pinned_apps {
            seen_apps.insert(app_id.to_lowercase());
            let app_windows: Vec<WindowInfo> = windows
                .iter()
                .filter(|w| {
                    let wm = w.wm_class.to_lowercase();
                    let target = app_id.to_lowercase();
                    wm.contains(&target) || target.contains(&wm)
                })
                .cloned()
                .collect();

            let is_running = !app_windows.is_empty();
            let is_active = app_windows.iter().any(|w| w.is_active);

            let btn = self.create_dock_item(app_id, is_running, is_active, app_windows);
            self.container.append(&btn);
        }

        // 2. Additional running apps that are not pinned
        for win in windows {
            let class_lower = win.wm_class.to_lowercase();
            if !seen_apps.contains(&class_lower) {
                seen_apps.insert(class_lower);
                let app_windows: Vec<WindowInfo> = windows
                    .iter()
                    .filter(|w| w.wm_class == win.wm_class)
                    .cloned()
                    .collect();

                let is_active = app_windows.iter().any(|w| w.is_active);
                let btn = self.create_dock_item(&win.wm_class, true, is_active, app_windows);
                self.container.append(&btn);
            }
        }
    }

    fn create_dock_item(
        &self,
        app_id: &str,
        is_running: bool,
        is_active: bool,
        windows: Vec<WindowInfo>,
    ) -> Button {
        let btn = Button::builder()
            .css_classes(["taskbar-button"])
            .build();

        let vbox = Box::new(Orientation::Vertical, 2);
        vbox.set_valign(gtk4::Align::Center);
        vbox.set_halign(gtk4::Align::Center);

        // Icon lookup
        let app_info = Self::find_app_info(app_id);
        let icon = if let Some(a) = &app_info {
            if let Some(gicon) = a.icon() {
                Image::from_gicon(&gicon)
            } else {
                Image::from_icon_name(app_id)
            }
        } else {
            Image::from_icon_name(app_id)
        };
        icon.set_pixel_size(26);

        vbox.append(&icon);

        // Win11 Pill Indicator
        if is_running {
            let indicator = Box::new(Orientation::Horizontal, 0);
            indicator.add_css_class("app-pill-indicator");
            if is_active {
                indicator.add_css_class("active");
            }
            vbox.append(&indicator);
        }

        btn.set_child(Some(&vbox));

        let backend = self.backend.clone();
        let id_owned = app_id.to_string();

        btn.connect_clicked(move |_| {
            if !windows.is_empty() {
                let first = &windows[0];
                if first.is_active && !first.is_minimized {
                    let _ = backend.minimize_window(&first.id);
                } else {
                    let _ = backend.focus_window(&first.id);
                }
            } else {
                // Launch app
                if let Some(app) = Self::find_app_info(&id_owned) {
                    let ctx = gtk4::gdk::Display::default().map(|d| d.app_launch_context());
                    let _ = app.launch(&[], ctx.as_ref());
                } else {
                    let _ = std::process::Command::new("gtk-launch").arg(&id_owned).spawn();
                }
            }
        });

        btn
    }
}
