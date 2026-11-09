use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Image, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use crate::services::network::NetworkService;
use crate::services::power::PowerService;

pub struct QuickToggles {
    grid: Grid,
    wifi_btn: Button,
    wifi_label: Label,
    power_btn: Button,
    power_label: Label,
    dnd_btn: Button,
    night_light_btn: Button,
    dnd_active: Rc<RefCell<bool>>,
    nl_active: Rc<RefCell<bool>>,
}

impl QuickToggles {
    pub fn new() -> Self {
        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .hexpand(true)
            .build();

        // Wi-Fi Toggle
        let (wifi_btn, wifi_label) = Self::create_pill("network-wireless-symbolic", "Wi-Fi");
        wifi_btn.connect_clicked(|_| {
            NetworkService::toggle_wifi();
        });
        grid.attach(&wifi_btn, 0, 0, 1, 1);

        // Bluetooth Toggle
        let (bt_btn, _) = Self::create_pill("bluetooth-active-symbolic", "Bluetooth");
        bt_btn.connect_clicked(|b| {
            b.has_css_class("active");
            if b.has_css_class("active") {
                b.remove_css_class("active");
            } else {
                b.add_css_class("active");
            }
        });
        grid.attach(&bt_btn, 1, 0, 1, 1);

        // Power Profile Toggle
        let (power_btn, power_label) = Self::create_pill("power-profile-balanced-symbolic", "Balanced");
        let pl_clone = power_label.clone();
        power_btn.connect_clicked(move |b| {
            let cur = pl_clone.text();
            let next = match cur.as_str() {
                "Balanced" => "Performance",
                "Performance" => "Power-saver",
                _ => "Balanced",
            };
            pl_clone.set_text(next);
            b.add_css_class("active");
            PowerService::set_power_profile(next.to_lowercase());
        });
        grid.attach(&power_btn, 0, 1, 1, 1);

        // Night Light Toggle
        let (night_light_btn, _) = Self::create_pill("night-light-symbolic", "Night Light");
        let nl_active = Rc::new(RefCell::new(false));
        let nl_clone = nl_active.clone();
        night_light_btn.connect_clicked(move |b| {
            let mut val = nl_clone.borrow_mut();
            *val = !*val;
            if *val {
                b.add_css_class("active");
            } else {
                b.remove_css_class("active");
            }
        });
        grid.attach(&night_light_btn, 1, 1, 1, 1);

        // Do Not Disturb (DND)
        let (dnd_btn, _) = Self::create_pill("notifications-disabled-symbolic", "Do Not Disturb");
        let dnd_active = Rc::new(RefCell::new(false));
        let dnd_clone = dnd_active.clone();
        dnd_btn.connect_clicked(move |b| {
            let mut val = dnd_clone.borrow_mut();
            *val = !*val;
            if *val {
                b.add_css_class("active");
            } else {
                b.remove_css_class("active");
            }
        });
        grid.attach(&dnd_btn, 0, 2, 2, 1);

        Self {
            grid,
            wifi_btn,
            wifi_label,
            power_btn,
            power_label,
            dnd_btn,
            night_light_btn,
            dnd_active,
            nl_active,
        }
    }

    fn create_pill(icon_name: &str, text: &str) -> (Button, Label) {
        let btn = Button::builder()
            .hexpand(true)
            .css_classes(["quick-toggle-pill"])
            .build();

        let hbox = Box::new(Orientation::Horizontal, 8);
        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(18);

        let label = Label::builder()
            .label(text)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        hbox.append(&icon);
        hbox.append(&label);
        btn.set_child(Some(&hbox));

        (btn, label)
    }

    pub fn widget(&self) -> &Grid {
        &self.grid
    }

    pub fn update_network(&self, is_connected: bool, is_wifi: bool, ssid: Option<&str>) {
        if is_wifi && is_connected {
            self.wifi_btn.add_css_class("active");
            if let Some(s) = ssid {
                self.wifi_label.set_text(s);
            } else {
                self.wifi_label.set_text("Connected");
            }
        } else if is_wifi {
            self.wifi_btn.remove_css_class("active");
            self.wifi_label.set_text("Wi-Fi");
        } else {
            self.wifi_btn.remove_css_class("active");
            self.wifi_label.set_text("Disabled");
        }
    }

    pub fn update_power(&self, profile: &str) {
        let display = match profile {
            "performance" => "Performance",
            "power-saver" => "Power Saver",
            _ => "Balanced",
        };
        self.power_label.set_text(display);
        self.power_btn.add_css_class("active");
    }
}
