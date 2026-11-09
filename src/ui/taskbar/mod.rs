pub mod app_dock;
pub mod systray_pill;

use gtk4::prelude::*;
use gtk4::{Box, Button, CenterBox, Image, Orientation, Window};
use std::sync::Arc;

use crate::backend::wayland_layer::LayerShellHelper;
use crate::backend::CompositorBackend;
use crate::config::Config;
use crate::services::{AudioState, NetworkState, PowerState};
use crate::ui::control_center::ControlCenterWindow;
use crate::ui::snap_layouts::SnapLayoutsMenu;
use crate::ui::start_menu::StartMenuWindow;
use app_dock::AppDock;
use systray_pill::SystrayPill;

pub struct TaskbarWindow {
    window: Window,
    dock: AppDock,
    systray: SystrayPill,
    snap_menu: SnapLayoutsMenu,
}

impl TaskbarWindow {
    pub fn new(
        app: &gtk4::Application,
        config: &Config,
        backend: Arc<dyn CompositorBackend>,
        start_menu: Arc<StartMenuWindow>,
        control_center: Arc<ControlCenterWindow>,
    ) -> Self {
        let is_top = config.taskbar.position.to_lowercase() == "top";
        let window = Window::builder()
            .application(app)
            .title("Kepler Taskbar")
            .decorated(false)
            .resizable(false)
            .css_classes(["kepler-surface"])
            .build();

        LayerShellHelper::init_taskbar(&window, config.taskbar.height, is_top);

        let center_box = CenterBox::new();
        center_box.add_css_class("kepler-taskbar");

        // Start Button
        let start_btn = Button::builder()
            .css_classes(["start-button"])
            .build();
        let start_icon = Image::from_icon_name("view-app-grid-symbolic");
        start_icon.set_pixel_size(24);
        start_btn.set_child(Some(&start_icon));

        let sm_clone = start_menu.clone();
        let cc_clone = control_center.clone();
        start_btn.connect_clicked(move |_| {
            cc_clone.hide();
            sm_clone.toggle();
        });

        // App Dock
        let dock = AppDock::new(&config.taskbar.pinned_apps, backend.clone());

        // Left section
        let left_box = Box::new(Orientation::Horizontal, 4);
        left_box.set_valign(gtk4::Align::Center);

        // Center section
        let mid_box = Box::new(Orientation::Horizontal, 4);
        mid_box.set_valign(gtk4::Align::Center);

        if config.taskbar.alignment.to_lowercase() == "left" {
            left_box.append(&start_btn);
            left_box.append(dock.widget());
        } else {
            mid_box.append(&start_btn);
            mid_box.append(dock.widget());
        }

        // Right section: Snap button + Systray Pill
        let right_box = Box::new(Orientation::Horizontal, 4);
        right_box.set_valign(gtk4::Align::Center);

        let snap_menu = SnapLayoutsMenu::new(backend.clone());
        let snap_btn = Button::builder()
            .icon_name("window-maximize-symbolic")
            .css_classes(["taskbar-button", "flat"])
            .build();
        snap_menu.attach_to(&snap_btn);

        let sm_ref = Arc::new(snap_menu);
        let sm_click = sm_ref.clone();
        snap_btn.connect_clicked(move |_| {
            sm_click.popup();
        });

        let systray = SystrayPill::new(&config.taskbar.clock_format);
        let sm_hide = start_menu.clone();
        let cc_toggle = control_center.clone();
        systray.widget().connect_clicked(move |_| {
            sm_hide.hide();
            cc_toggle.toggle();
        });

        right_box.append(&snap_btn);
        right_box.append(systray.widget());

        center_box.set_start_widget(Some(&left_box));
        center_box.set_center_widget(Some(&mid_box));
        center_box.set_end_widget(Some(&right_box));

        window.set_child(Some(&center_box));
        window.present();

        Self {
            window,
            dock,
            systray,
            snap_menu: (*sm_ref).clone(),
        }
    }

    pub fn refresh_dock(&self, backend: &dyn CompositorBackend) {
        self.dock.refresh(&backend.get_windows());
    }

    pub fn update_network(&self, net: &NetworkState) {
        self.systray.update_network(net);
    }

    pub fn update_audio(&self, audio: &AudioState) {
        self.systray.update_audio(audio);
    }

    pub fn update_power(&self, power: &PowerState) {
        self.systray.update_power(power);
    }
}
