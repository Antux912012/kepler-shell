// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Manages application dock buttons, pinned launchers, and running indicators.
 * Architecture Position: Child component of KeplerTaskbar in taskbar.js.
 * Concurrency Constraints: Executes on the GNOME Shell main loop thread.
 */

import Clutter from 'gi://Clutter';
import St from 'gi://St';
import Shell from 'gi://Shell';

/**
 * Renders the application dock with Windows 11 style indicators.
 */
export class AppDock {
    /**
     * Initializes the dock container and registers GNOME Shell window listeners.
     * @param {Gio.Settings} settings - The Kepler Shell GSettings instance.
     */
    constructor(settings) {
        this.settings = settings;
        this.actor = new St.BoxLayout({
            style_class: 'kepler-app-dock',
            vertical: false,
            x_align: Clutter.ActorAlign.CENTER,
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._signals = [];
        this._appSys = Shell.AppSystem.get_default();

        this._signals.push(
            this._appSys.connect('app-state-changed', () => this.refresh()),
            global.display.connect('notify::focus-window', () => this.refresh()),
            global.display.connect('window-created', () => this.refresh()),
            this.settings.connect('changed::pinned-apps', () => this.refresh()),
            this.settings.connect('changed::show-running-apps', () => this.refresh()),
            this.settings.connect('changed::icon-size', () => this.refresh())
        );

        this.refresh();
    }

    /**
     * Rebuilds all dock items from pinned applications and active window state.
     */
    refresh() {
        this.actor.destroy_all_children();

        const pinnedIds = this.settings.get_strv('pinned-apps');
        const showRunning = this.settings.get_boolean('show-running-apps');
        const iconSize = this.settings.get_int('icon-size');
        const focusWindow = global.display.focus_window;
        const seenApps = new Set();

        // 1. Render Pinned Applications
        for (const appId of pinnedIds) {
            const app = this._appSys.lookup_app(appId);
            if (!app) continue;

            seenApps.add(app.get_id());
            const windows = app.get_windows();
            const isRunning = windows.length > 0;
            const isActive = focusWindow && windows.some(w => w === focusWindow);

            const btn = this._createDockButton(app, isRunning, isActive, iconSize);
            this.actor.add_child(btn);
        }

        // 2. Render Unpinned Active Applications
        if (showRunning) {
            const runningApps = this._appSys.get_running();
            for (const app of runningApps) {
                if (seenApps.has(app.get_id())) continue;

                const windows = app.get_windows();
                const isActive = focusWindow && windows.some(w => w === focusWindow);
                const btn = this._createDockButton(app, true, isActive, iconSize);
                this.actor.add_child(btn);
            }
        }
    }

    /**
     * Creates a single application dock launcher button.
     * @param {Shell.App} app - The GNOME Shell application instance.
     * @param {boolean} isRunning - True if the application has open windows.
     * @param {boolean} isActive - True if the application owns the focused window.
     * @param {number} iconSize - Width and height of the icon texture in pixels.
     * @returns {St.Button} The dock button actor.
     */
    _createDockButton(app, isRunning, isActive, iconSize) {
        const btn = new St.Button({
            style_class: 'kepler-dock-item',
            can_focus: true,
            reactive: true,
        });

        const box = new St.BoxLayout({
            vertical: true,
            x_align: Clutter.ActorAlign.CENTER,
            y_align: Clutter.ActorAlign.CENTER,
        });

        const icon = app.create_icon_texture(iconSize);
        box.add_child(icon);

        if (isRunning) {
            const pill = new St.Widget({
                style_class: isActive ? 'kepler-pill-indicator active' : 'kepler-pill-indicator',
            });
            box.add_child(pill);
        }

        btn.set_child(box);

        btn.connect('clicked', () => {
            const windows = app.get_windows();
            if (windows.length === 0) {
                app.open_new_window(-1);
            } else {
                const win = windows[0];
                if (win === global.display.focus_window && !win.minimized) {
                    win.minimize();
                } else {
                    if (win.minimized) win.unminimize();
                    win.activate(global.get_current_time());
                }
            }
        });

        return btn;
    }

    /**
     * Disconnects all registered signal handlers and destroys actors.
     */
    destroy() {
        for (const sig of this._signals) {
            try {
                this._appSys.disconnect(sig);
                global.display.disconnect(sig);
                this.settings.disconnect(sig);
            } catch (_) {}
        }
        this._signals = [];
        this.actor.destroy();
    }
}
