// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Manages the main 48px taskbar container, layout, and alignment.
 * Architecture Position: Instantiated by KeplerShellExtension (extension.js), parent of dock and systray.
 * Concurrency Constraints: Executes on the GNOME Shell main loop thread.
 */

import Clutter from 'gi://Clutter';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { AppDock } from './appDock.js';
import { SystrayPill } from './systrayPill.js';

/**
 * Manages the Kepler Shell main taskbar panel.
 */
export class KeplerTaskbar {
    /**
     * Initializes the taskbar actor, builds child widgets, and registers layout signals.
     * @param {Gio.Settings} settings - The Kepler Shell GSettings instance.
     * @param {Object} startMenu - The StartMenuFlyout instance.
     * @param {Object} controlCenter - The ControlCenterFlyout instance.
     * @param {Object} snapMenu - The SnapLayoutsMenu instance.
     */
    constructor(settings, startMenu, controlCenter, snapMenu) {
        this.settings = settings;
        this.startMenu = startMenu;
        this.controlCenter = controlCenter;
        this.snapMenu = snapMenu;

        this.actor = new St.BoxLayout({
            style_class: 'kepler-taskbar-panel',
            reactive: true,
            can_focus: false,
        });

        this._signals = [];
        this._buildUI();

        Main.layoutManager.addTopChrome(this.actor);
        this.updatePosition();

        this._signals.push(
            this.settings.connect('changed::taskbar-position', () => this.updatePosition()),
            this.settings.connect('changed::taskbar-height', () => this.updatePosition()),
            this.settings.connect('changed::taskbar-alignment', () => this._buildUI()),
            global.display.connect('workareas-changed', () => this.updatePosition())
        );
    }

    /**
     * Constructs the taskbar layout with Start Button, App Dock, and Systray.
     */
    _buildUI() {
        this.actor.destroy_all_children();

        const alignment = this.settings.get_string('taskbar-alignment');

        // Start Button
        this._startBtn = new St.Button({
            style_class: 'kepler-start-button',
            can_focus: true,
            reactive: true,
            y_align: Clutter.ActorAlign.CENTER,
        });
        const startIcon = new St.Icon({
            icon_name: 'view-app-grid-symbolic',
            icon_size: 22,
        });
        this._startBtn.set_child(startIcon);
        this._startBtn.connect('clicked', () => {
            const isTop = this.settings.get_string('taskbar-position') === 'top';
            this.controlCenter.hide();
            this.startMenu.toggleAt(this._startBtn, isTop, alignment);
        });

        // App Dock
        this._dock = new AppDock(this.settings);

        // Status Indicator Pill
        this._systray = new SystrayPill(this.settings);
        this._systray.actor.connect('clicked', () => {
            const isTop = this.settings.get_string('taskbar-position') === 'top';
            this.startMenu.hide();
            this.controlCenter.toggleAt(this._systray.actor, isTop);
        });

        // Snap Layouts Button
        this._snapBtn = new St.Button({
            style_class: 'kepler-dock-item',
            can_focus: true,
            reactive: true,
            y_align: Clutter.ActorAlign.CENTER,
            style: 'margin-right: 4px;',
        });
        this._snapBtn.set_child(new St.Icon({
            icon_name: 'window-maximize-symbolic',
            icon_size: 16,
        }));
        this._snapBtn.connect('clicked', () => {
            const isTop = this.settings.get_string('taskbar-position') === 'top';
            this.snapMenu.toggleAt(this._snapBtn, isTop);
        });

        // Left Container Box
        const leftBox = new St.BoxLayout({
            vertical: false,
            x_align: Clutter.ActorAlign.START,
            x_expand: true,
        });

        // Center Container Box
        const centerBox = new St.BoxLayout({
            vertical: false,
            x_align: Clutter.ActorAlign.CENTER,
        });

        // Right Container Box
        const rightBox = new St.BoxLayout({
            vertical: false,
            x_align: Clutter.ActorAlign.END,
            x_expand: true,
            y_align: Clutter.ActorAlign.CENTER,
        });

        if (alignment === 'left') {
            leftBox.add_child(this._startBtn);
            leftBox.add_child(this._dock.actor);
        } else {
            centerBox.add_child(this._startBtn);
            centerBox.add_child(this._dock.actor);
        }

        if (this.settings.get_boolean('enable-snap-layouts')) {
            rightBox.add_child(this._snapBtn);
        }
        rightBox.add_child(this._systray.actor);

        this.actor.add_child(leftBox);
        this.actor.add_child(centerBox);
        this.actor.add_child(rightBox);
    }

    /**
     * Updates the size and screen coordinates of the taskbar based on user preferences.
     */
    updatePosition() {
        const primary = Main.layoutManager.primaryMonitor;
        if (!primary) return;

        const isTop = this.settings.get_string('taskbar-position') === 'top';
        const height = this.settings.get_int('taskbar-height');

        this.actor.set_size(primary.width, height);
        const posY = isTop ? primary.y : primary.y + primary.height - height;
        this.actor.set_position(primary.x, posY);

        if (isTop) {
            this.actor.add_style_class_name('top-dock');
        } else {
            this.actor.remove_style_class_name('top-dock');
        }
    }

    /**
     * Disconnects all registered signal handlers, removes chrome, and destroys actors.
     */
    destroy() {
        for (const sig of this._signals) {
            try {
                this.settings.disconnect(sig);
                global.display.disconnect(sig);
            } catch (_) {}
        }
        this._signals = [];

        if (this._dock) this._dock.destroy();
        if (this._systray) this._systray.destroy();

        Main.layoutManager.removeChrome(this.actor);
        this.actor.destroy();
    }
}
