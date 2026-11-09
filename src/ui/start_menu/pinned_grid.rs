use gio::prelude::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, FlowBox, Image, Label, Orientation, ScrolledWindow};

pub struct PinnedGrid {
    scrolled: ScrolledWindow,
    flow_box: FlowBox,
}

impl PinnedGrid {
    pub fn new(pinned_apps: &[String]) -> Self {
        let flow_box = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(6)
            .min_children_per_line(4)
            .row_spacing(8)
            .column_spacing(8)
            .hexpand(true)
            .build();

        for app_id in pinned_apps {
            let item = Self::create_app_item(app_id);
            flow_box.append(&item);
        }

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(200)
            .max_content_height(280)
            .child(&flow_box)
            .build();

        Self { scrolled, flow_box }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.scrolled
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

    fn create_app_item(app_id: &str) -> Button {
        let app_info = Self::find_app_info(app_id);
        let name = app_info
            .as_ref()
            .map(|a| a.name().to_string())
            .unwrap_or_else(|| app_id.split('.').last().unwrap_or(app_id).to_string());

        let icon = if let Some(a) = &app_info {
            if let Some(gicon) = a.icon() {
                Image::from_gicon(&gicon)
            } else {
                Image::from_icon_name("application-x-executable")
            }
        } else {
            Image::from_icon_name("application-x-executable")
        };
        icon.set_pixel_size(36);

        let label = Label::builder()
            .label(&name)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(10)
            .build();

        let vbox = Box::new(Orientation::Vertical, 4);
        vbox.set_halign(gtk4::Align::Center);
        vbox.append(&icon);
        vbox.append(&label);

        let btn = Button::builder()
            .child(&vbox)
            .css_classes(["pinned-app-item"])
            .build();

        let id_owned = app_id.to_string();
        btn.connect_clicked(move |_| {
            if let Some(app) = Self::find_app_info(&id_owned) {
                let context = gtk4::gdk::Display::default().map(|d| d.app_launch_context());
                let _ = app.launch(&[], context.as_ref());
            } else {
                let _ = std::process::Command::new("gtk-launch")
                    .arg(&id_owned)
                    .spawn();
            }
        });

        btn
    }
}
