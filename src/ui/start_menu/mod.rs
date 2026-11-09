pub mod pinned_grid;
pub mod power_footer;
pub mod recommended;
pub mod search_bar;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Separator, Window};

use crate::backend::wayland_layer::LayerShellHelper;
use crate::config::Config;
use pinned_grid::PinnedGrid;
use power_footer::PowerFooter;
use recommended::RecommendedSection;
use search_bar::StartSearchBar;

pub struct StartMenuWindow {
    window: Window,
    search_bar: StartSearchBar,
}

impl StartMenuWindow {
    pub fn new(app: &gtk4::Application, config: &Config) -> Self {
        let window = Window::builder()
            .application(app)
            .title("Kepler Start Menu")
            .decorated(false)
            .resizable(false)
            .css_classes(["kepler-surface"])
            .build();

        LayerShellHelper::init_flyout(
            &window,
            config.start_menu.width,
            config.start_menu.height,
            config.taskbar.height + config.theme.flyout_margin,
            true,
        );

        let root_box = Box::new(Orientation::Vertical, 12);
        root_box.add_css_class("kepler-flyout");

        let search_bar = StartSearchBar::new(&config.start_menu.search_placeholder);
        root_box.append(search_bar.widget());

        let pinned = PinnedGrid::new(&config.taskbar.pinned_apps);
        root_box.append(pinned.widget());

        if config.start_menu.show_recommended {
            root_box.append(&Separator::new(Orientation::Horizontal));
            let rec = RecommendedSection::new(config.start_menu.max_recent_files);
            root_box.append(rec.widget());
        }

        root_box.append(&Separator::new(Orientation::Horizontal));
        let footer = PowerFooter::new();
        root_box.append(footer.widget());

        window.set_child(Some(&root_box));
        window.set_visible(false);

        // Close on Escape key
        let key_controller = gtk4::EventControllerKey::new();
        let win_clone = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                win_clone.set_visible(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        Self { window, search_bar }
    }

    pub fn toggle(&self) {
        let is_vis = self.window.is_visible();
        if !is_vis {
            self.window.set_visible(true);
            self.search_bar.focus();
        } else {
            self.window.set_visible(false);
            self.search_bar.clear();
        }
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
        self.search_bar.clear();
    }
}
