use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, Popover};

pub struct PowerFooter {
    container: Box,
}

impl PowerFooter {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 12);
        container.add_css_class("power-footer");

        // User profile pill
        let user_box = Box::new(Orientation::Horizontal, 8);
        user_box.set_hexpand(true);

        let avatar = Image::from_icon_name("avatar-default-symbolic");
        avatar.set_pixel_size(28);
        avatar.add_css_class("avatar-badge");

        let username = std::env::var("USER").unwrap_or_else(|_| "User".to_string());
        let user_lbl = Label::builder()
            .label(&username)
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .build();

        user_box.append(&avatar);
        user_box.append(&user_lbl);

        // Power button with popover
        let power_btn = Button::builder()
            .icon_name("system-shutdown-symbolic")
            .css_classes(["flat"])
            .build();

        let popover = Popover::new();
        let menu_box = Box::new(Orientation::Vertical, 4);

        let lock_btn = Self::create_action_btn("Lock", "system-lock-screen-symbolic", || {
            let _ = std::process::Command::new("loginctl")
                .arg("lock-session")
                .spawn();
        });

        let sleep_btn = Self::create_action_btn("Sleep", "system-suspend-symbolic", || {
            let _ = std::process::Command::new("systemctl")
                .arg("suspend")
                .spawn();
        });

        let restart_btn = Self::create_action_btn("Restart", "system-reboot-symbolic", || {
            let _ = std::process::Command::new("systemctl")
                .arg("reboot")
                .spawn();
        });

        let shutdown_btn = Self::create_action_btn("Shut Down", "system-shutdown-symbolic", || {
            let _ = std::process::Command::new("systemctl")
                .arg("poweroff")
                .spawn();
        });

        menu_box.append(&lock_btn);
        menu_box.append(&sleep_btn);
        menu_box.append(&restart_btn);
        menu_box.append(&shutdown_btn);
        popover.set_child(Some(&menu_box));
        popover.set_parent(&power_btn);

        power_btn.connect_clicked(move |_| {
            popover.popup();
        });

        container.append(&user_box);
        container.append(&power_btn);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn create_action_btn<F: Fn() + 'static>(title: &str, icon: &str, action: F) -> Button {
        let btn = Button::builder()
            .css_classes(["flat"])
            .build();

        let hbox = Box::new(Orientation::Horizontal, 8);
        let img = Image::from_icon_name(icon);
        let lbl = Label::new(Some(title));

        hbox.append(&img);
        hbox.append(&lbl);
        btn.set_child(Some(&hbox));

        btn.connect_clicked(move |_| {
            action();
        });

        btn
    }
}
