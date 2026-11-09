use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation, Popover};
use std::sync::Arc;

use crate::backend::{CompositorBackend, SnapPosition};

#[derive(Clone)]
pub struct SnapLayoutsMenu {
    popover: Popover,
    backend: Arc<dyn CompositorBackend>,
}

impl SnapLayoutsMenu {
    pub fn new(backend: Arc<dyn CompositorBackend>) -> Self {
        let popover = Popover::new();
        popover.add_css_class("snap-overlay-window");

        let root_box = Box::new(Orientation::Vertical, 10);
        let header = Label::builder()
            .label("Snap Window Layout")
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .build();
        root_box.append(&header);

        let grid = Grid::builder()
            .row_spacing(12)
            .column_spacing(12)
            .build();

        // Preset 1: 50 / 50 Half Split
        let half_box = Box::new(Orientation::Horizontal, 4);
        let btn_l50 = Self::create_snap_button("Left 1/2", SnapPosition::Left, backend.clone(), &popover);
        let btn_r50 = Self::create_snap_button("Right 1/2", SnapPosition::Right, backend.clone(), &popover);
        half_box.append(&btn_l50);
        half_box.append(&btn_r50);
        grid.attach(&half_box, 0, 0, 1, 1);

        // Preset 2: 2x2 Quadrants
        let quad_grid = Grid::builder().row_spacing(4).column_spacing(4).build();
        let btn_tl = Self::create_snap_button("TL", SnapPosition::TopLeft, backend.clone(), &popover);
        let btn_tr = Self::create_snap_button("TR", SnapPosition::TopRight, backend.clone(), &popover);
        let btn_bl = Self::create_snap_button("BL", SnapPosition::BottomLeft, backend.clone(), &popover);
        let btn_br = Self::create_snap_button("BR", SnapPosition::BottomRight, backend.clone(), &popover);
        quad_grid.attach(&btn_tl, 0, 0, 1, 1);
        quad_grid.attach(&btn_tr, 1, 0, 1, 1);
        quad_grid.attach(&btn_bl, 0, 1, 1, 1);
        quad_grid.attach(&btn_br, 1, 1, 1, 1);
        grid.attach(&quad_grid, 1, 0, 1, 1);

        // Preset 3: Maximize
        let max_box = Box::new(Orientation::Horizontal, 4);
        let btn_max = Self::create_snap_button("Full", SnapPosition::Maximize, backend.clone(), &popover);
        btn_max.set_hexpand(true);
        max_box.append(&btn_max);
        grid.attach(&max_box, 0, 1, 2, 1);

        root_box.append(&grid);
        popover.set_child(Some(&root_box));

        Self { popover, backend }
    }

    fn create_snap_button(
        label: &str,
        pos: SnapPosition,
        backend: Arc<dyn CompositorBackend>,
        popover: &Popover,
    ) -> Button {
        let btn = Button::builder()
            .label(label)
            .css_classes(["snap-zone-btn"])
            .build();

        let pop_clone = popover.clone();
        btn.connect_clicked(move |_| {
            let windows = backend.get_windows();
            if let Some(active) = windows.iter().find(|w| w.is_active) {
                let _ = backend.snap_window(&active.id, pos);
            }
            pop_clone.popdown();
        });

        btn
    }

    pub fn attach_to<W: IsA<gtk4::Widget>>(&self, parent: &W) {
        self.popover.set_parent(parent);
    }

    pub fn popup(&self) {
        self.popover.popup();
    }
}
