use chrono::Local;
use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation};
use std::time::Duration;

use crate::services::{AudioState, NetworkState, PowerState};

pub struct SystrayPill {
    button: Button,
    net_icon: Image,
    audio_icon: Image,
    power_icon: Image,
    clock_label: Label,
    clock_format: String,
}

impl SystrayPill {
    pub fn new(clock_format: &str) -> Self {
        let button = Button::builder()
            .css_classes(["systray-pill"])
            .build();

        let hbox = Box::new(Orientation::Horizontal, 8);
        hbox.set_valign(gtk4::Align::Center);

        let net_icon = Image::from_icon_name("network-wireless-symbolic");
        net_icon.set_pixel_size(16);

        let audio_icon = Image::from_icon_name("audio-volume-high-symbolic");
        audio_icon.set_pixel_size(16);

        let power_icon = Image::from_icon_name("battery-level-100-symbolic");
        power_icon.set_pixel_size(16);

        let clock_label = Label::builder()
            .label(&Local::now().format(clock_format).to_string())
            .css_classes(["systray-clock"])
            .justify(gtk4::Justification::Center)
            .build();

        hbox.append(&net_icon);
        hbox.append(&audio_icon);
        hbox.append(&power_icon);
        hbox.append(&clock_label);

        button.set_child(Some(&hbox));

        // Clock timer
        let clk_lbl = clock_label.clone();
        let fmt = clock_format.to_string();
        glib::timeout_add_local(Duration::from_secs(1), move || {
            clk_lbl.set_text(&Local::now().format(&fmt).to_string());
            glib::ControlFlow::Continue
        });

        Self {
            button,
            net_icon,
            audio_icon,
            power_icon,
            clock_label,
            clock_format: clock_format.to_string(),
        }
    }

    pub fn widget(&self) -> &Button {
        &self.button
    }

    pub fn update_network(&self, net: &NetworkState) {
        let icon = if net.is_connected {
            "network-wireless-signal-excellent-symbolic"
        } else if net.is_wifi_enabled {
            "network-wireless-offline-symbolic"
        } else {
            "network-wireless-disabled-symbolic"
        };
        self.net_icon.set_icon_name(Some(icon));
    }

    pub fn update_audio(&self, audio: &AudioState) {
        let icon = if audio.is_muted || audio.volume == 0 {
            "audio-volume-muted-symbolic"
        } else if audio.volume < 33 {
            "audio-volume-low-symbolic"
        } else if audio.volume < 66 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        };
        self.audio_icon.set_icon_name(Some(icon));
    }

    pub fn update_power(&self, power: &PowerState) {
        if !power.has_battery {
            self.power_icon.set_visible(false);
            return;
        }
        self.power_icon.set_visible(true);

        let icon = if power.is_charging {
            "battery-level-100-charging-symbolic"
        } else {
            let level = (power.percentage / 10) * 10;
            match level {
                0..=20 => "battery-level-10-symbolic",
                21..=40 => "battery-level-30-symbolic",
                41..=60 => "battery-level-50-symbolic",
                61..=80 => "battery-level-70-symbolic",
                _ => "battery-level-100-symbolic",
            }
        };
        self.power_icon.set_icon_name(Some(icon));
    }
}
