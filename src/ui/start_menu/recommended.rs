use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, ListBox, Orientation, RecentManager};

pub struct RecommendedSection {
    container: Box,
    list_box: ListBox,
}

impl RecommendedSection {
    pub fn new(max_items: usize) -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.set_margin_top(8);

        let header = Label::builder()
            .label("Recommended")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        container.append(&header);

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        Self::populate_recent_files(&list_box, max_items);
        container.append(&list_box);

        Self { container, list_box }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn populate_recent_files(list_box: &ListBox, max_items: usize) {
        let manager = RecentManager::default();
        let items = manager.items();

        let mut count = 0;
        for info in items.iter() {
            if count >= max_items {
                break;
            }
            if !info.exists() {
                continue;
            }

            let display_name = info.display_name().to_string();
            let uri = info.uri().to_string();

            let row_btn = Button::builder()
                .css_classes(["recommended-file-item", "flat"])
                .build();

            let hbox = Box::new(Orientation::Horizontal, 10);
            let icon = Image::from_icon_name("text-x-generic");
            icon.set_pixel_size(24);

            let vbox = Box::new(Orientation::Vertical, 2);
            let name_lbl = Label::builder()
                .label(&display_name)
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            let uri_lbl = Label::builder()
                .label(&uri)
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                .css_classes(["dim-label", "caption"])
                .build();

            vbox.append(&name_lbl);
            vbox.append(&uri_lbl);

            hbox.append(&icon);
            hbox.append(&vbox);
            row_btn.set_child(Some(&hbox));

            let uri_clone = uri.clone();
            row_btn.connect_clicked(move |_| {
                let _ = gio::AppInfo::launch_default_for_uri(&uri_clone, None::<&gio::AppLaunchContext>);
            });

            list_box.append(&row_btn);
            count += 1;
        }

        if count == 0 {
            let empty_lbl = Label::builder()
                .label("No recent files")
                .css_classes(["dim-label"])
                .halign(gtk4::Align::Start)
                .build();
            list_box.append(&empty_lbl);
        }
    }
}
