use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Orientation, Scale};
use std::cell::Cell;
use std::rc::Rc;

use crate::services::audio::AudioService;

pub struct ControlSliders {
    container: Box,
    vol_scale: Scale,
    vol_btn: Button,
    is_updating: Rc<Cell<bool>>,
}

impl ControlSliders {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        // Volume row
        let vol_row = Box::new(Orientation::Horizontal, 8);
        let vol_btn = Button::builder()
            .icon_name("audio-volume-high-symbolic")
            .css_classes(["flat"])
            .build();

        vol_btn.connect_clicked(|_| {
            AudioService::toggle_mute();
        });

        let vol_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        vol_scale.set_hexpand(true);
        vol_scale.set_value(50.0);
        vol_scale.add_css_class("fluent-slider");

        let is_updating = Rc::new(Cell::new(false));
        let updating_clone = is_updating.clone();

        vol_scale.connect_value_changed(move |scale| {
            if !updating_clone.get() {
                let val = scale.value() as u32;
                AudioService::set_volume(val);
            }
        });

        vol_row.append(&vol_btn);
        vol_row.append(&vol_scale);

        // Brightness row
        let bright_row = Box::new(Orientation::Horizontal, 8);
        let bright_icon = Image::from_icon_name("display-brightness-symbolic");
        bright_icon.set_pixel_size(18);

        let bright_scale = Scale::with_range(Orientation::Horizontal, 10.0, 100.0, 1.0);
        bright_scale.set_hexpand(true);
        bright_scale.set_value(80.0);
        bright_scale.add_css_class("fluent-slider");

        bright_scale.connect_value_changed(|scale| {
            let val = scale.value() as u32;
            glib::MainContext::default().spawn_local(async move {
                let _ = tokio::process::Command::new("brightnessctl")
                    .args(["set", &format!("{}%", val)])
                    .output()
                    .await;
            });
        });

        bright_row.append(&bright_icon);
        bright_row.append(&bright_scale);

        container.append(&vol_row);
        container.append(&bright_row);

        Self {
            container,
            vol_scale,
            vol_btn,
            is_updating,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn update_volume(&self, volume: u32, is_muted: bool) {
        self.is_updating.set(true);
        self.vol_scale.set_value(volume as f64);
        let icon = if is_muted || volume == 0 {
            "audio-volume-muted-symbolic"
        } else if volume < 33 {
            "audio-volume-low-symbolic"
        } else if volume < 66 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        };
        self.vol_btn.set_icon_name(icon);
        self.is_updating.set(false);
    }
}
