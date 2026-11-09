use gtk4::prelude::*;
use gtk4::{Box, Image, Label, LevelBar, Orientation, Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::backend::wayland_layer::LayerShellHelper;

pub struct OsdWindow {
    window: Window,
    icon: Image,
    label: Label,
    bar: LevelBar,
    timeout_source: Rc<RefCell<Option<glib::SourceId>>>,
}

impl OsdWindow {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = Window::builder()
            .application(app)
            .title("Kepler OSD")
            .decorated(false)
            .resizable(false)
            .css_classes(["kepler-surface"])
            .build();

        LayerShellHelper::init_osd(&window, 240, 42);

        let container = Box::new(Orientation::Horizontal, 12);
        container.add_css_class("osd-pill");
        container.set_valign(gtk4::Align::Center);
        container.set_halign(gtk4::Align::Center);

        let icon = Image::from_icon_name("audio-volume-medium");
        icon.set_pixel_size(20);

        let label = Label::new(Some("50%"));
        label.set_width_chars(5);

        let bar = LevelBar::builder()
            .min_value(0.0)
            .max_value(1.0)
            .value(0.5)
            .hexpand(true)
            .build();
        bar.set_size_request(100, 4);

        container.append(&icon);
        container.append(&bar);
        container.append(&label);

        window.set_child(Some(&container));
        window.set_visible(false);

        Self {
            window,
            icon,
            label,
            bar,
            timeout_source: Rc::new(RefCell::new(None)),
        }
    }

    pub fn show_value(&self, icon_name: &str, text: &str, fraction: f64) {
        self.icon.set_icon_name(Some(icon_name));
        self.label.set_text(text);
        self.bar.set_value(fraction);

        self.window.set_visible(true);

        if let Some(source) = self.timeout_source.borrow_mut().take() {
            source.remove();
        }

        let win_clone = self.window.clone();
        let timeout_holder = self.timeout_source.clone();
        let source_id = glib::timeout_add_local_once(Duration::from_millis(1800), move || {
            win_clone.set_visible(false);
            timeout_holder.borrow_mut().take();
        });

        *self.timeout_source.borrow_mut() = Some(source_id);
    }
}
