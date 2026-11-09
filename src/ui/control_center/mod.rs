pub mod quick_toggles;
pub mod sliders;

use gtk4::prelude::*;
use gtk4::{Box, Calendar, Label, Orientation, Separator, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::wayland_layer::LayerShellHelper;
use crate::config::Config;
use crate::services::mpris::{MprisService, MprisState};
use crate::services::{AudioState, NetworkState, PowerState};
use quick_toggles::QuickToggles;
use sliders::ControlSliders;

pub struct ControlCenterWindow {
    window: Window,
    quick_toggles: QuickToggles,
    sliders: ControlSliders,
    media_box: Box,
    media_title: Label,
    media_artist: Label,
    current_mpris: Rc<RefCell<Option<MprisState>>>,
}

impl ControlCenterWindow {
    pub fn new(app: &gtk4::Application, config: &Config) -> Self {
        let window = Window::builder()
            .application(app)
            .title("Kepler Control Center")
            .decorated(false)
            .resizable(false)
            .css_classes(["kepler-surface"])
            .build();

        LayerShellHelper::init_flyout(
            &window,
            config.control_center.width,
            520,
            config.taskbar.height + config.theme.flyout_margin,
            false,
        );

        let root_box = Box::new(Orientation::Vertical, 12);
        root_box.add_css_class("kepler-flyout");

        let quick_toggles = QuickToggles::new();
        let sliders = ControlSliders::new();

        root_box.append(quick_toggles.widget());
        root_box.append(&Separator::new(Orientation::Horizontal));
        root_box.append(sliders.widget());

        // MPRIS Media Card
        let media_box = Box::new(Orientation::Vertical, 4);
        media_box.add_css_class("recommended-file-item");
        let media_title = Label::builder()
            .label("No Media Playing")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        let media_artist = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .build();

        let media_controls = Box::new(Orientation::Horizontal, 8);
        media_controls.set_halign(gtk4::Align::Center);

        let current_mpris: Rc<RefCell<Option<MprisState>>> = Rc::new(RefCell::new(None));

        let btn_prev = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
        let m_prev = current_mpris.clone();
        btn_prev.connect_clicked(move |_| {
            if let Some(m) = m_prev.borrow().as_ref() {
                MprisService::previous(m.player_bus.clone());
            }
        });

        let btn_play = gtk4::Button::from_icon_name("media-playback-start-symbolic");
        let m_play = current_mpris.clone();
        btn_play.connect_clicked(move |_| {
            if let Some(m) = m_play.borrow().as_ref() {
                MprisService::play_pause(m.player_bus.clone());
            }
        });

        let btn_next = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        let m_next = current_mpris.clone();
        btn_next.connect_clicked(move |_| {
            if let Some(m) = m_next.borrow().as_ref() {
                MprisService::next(m.player_bus.clone());
            }
        });

        media_controls.append(&btn_prev);
        media_controls.append(&btn_play);
        media_controls.append(&btn_next);

        media_box.append(&media_title);
        media_box.append(&media_artist);
        media_box.append(&media_controls);

        root_box.append(&Separator::new(Orientation::Horizontal));
        root_box.append(&media_box);

        if config.control_center.show_calendar {
            root_box.append(&Separator::new(Orientation::Horizontal));
            let cal = Calendar::new();
            cal.set_hexpand(true);
            root_box.append(&cal);
        }

        window.set_child(Some(&root_box));
        window.set_visible(false);

        Self {
            window,
            quick_toggles,
            sliders,
            media_box,
            media_title,
            media_artist,
            current_mpris,
        }
    }

    pub fn toggle(&self) {
        let is_vis = self.window.is_visible();
        self.window.set_visible(!is_vis);
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn update_audio(&self, audio: &AudioState) {
        self.sliders.update_volume(audio.volume, audio.is_muted);
    }

    pub fn update_network(&self, net: &NetworkState) {
        self.quick_toggles.update_network(
            net.is_connected,
            net.is_wifi_enabled,
            net.ssid.as_deref(),
        );
    }

    pub fn update_power(&self, power: &PowerState) {
        self.quick_toggles.update_power(&power.power_profile);
    }

    pub fn update_mpris(&self, mpris: Option<MprisState>) {
        if let Some(m) = &mpris {
            self.media_title.set_text(&m.title);
            self.media_artist.set_text(&m.artist);
            self.media_box.set_visible(true);
        } else {
            self.media_box.set_visible(false);
        }
        *self.current_mpris.borrow_mut() = mpris;
    }
}
