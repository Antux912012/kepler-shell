// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Displays a unified indicator pill for network, volume, battery, and clock.
 * Architecture Position: Child component in KeplerTaskbar (taskbar.js), triggers ControlCenterFlyout.
 * Concurrency Constraints: Runs on the GNOME Shell main loop thread. Updates clock once per second.
 */

import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import St from 'gi://St';

/**
 * Renders the compact status indicator pill on the taskbar.
 */
export class SystrayPill {
    /**
     * Initializes the status indicator pill actor and registers the clock timer.
     * @param {Gio.Settings} settings - The Kepler Shell GSettings instance.
     */
    constructor(settings) {
        this.settings = settings;
        this.actor = new St.Button({
            style_class: 'kepler-systray-pill',
            can_focus: true,
            reactive: true,
            y_align: Clutter.ActorAlign.CENTER,
        });

        const box = new St.BoxLayout({
            vertical: false,
            x_align: Clutter.ActorAlign.CENTER,
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._netIcon = new St.Icon({
            icon_name: 'network-wireless-signal-excellent-symbolic',
            icon_size: 15,
            style: 'margin-right: 6px;',
        });

        this._volIcon = new St.Icon({
            icon_name: 'audio-volume-high-symbolic',
            icon_size: 15,
            style: 'margin-right: 6px;',
        });

        this._batIcon = new St.Icon({
            icon_name: 'battery-level-100-symbolic',
            icon_size: 15,
            style: 'margin-right: 8px;',
        });

        this._clockLabel = new St.Label({
            style_class: 'kepler-clock-label',
            y_align: Clutter.ActorAlign.CENTER,
        });

        box.add_child(this._netIcon);
        box.add_child(this._volIcon);
        box.add_child(this._batIcon);
        box.add_child(this._clockLabel);
        this.actor.set_child(box);

        this._updateClock();
        this._timeoutId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 1, () => {
            this._updateClock();
            return GLib.SOURCE_CONTINUE;
        });

        this._sig = this.settings.connect('changed::clock-format', () => this._updateClock());
    }

    /**
     * Updates the clock label text with the formatted current local time.
     */
    _updateClock() {
        const format = this.settings.get_string('clock-format') || '%H:%M\n%d/%m/%Y';
        const now = GLib.DateTime.new_now_local();
        this._clockLabel.text = now.format(format) || now.format('%H:%M');
    }

    /**
     * Removes the clock timer and destroys all actors.
     */
    destroy() {
        if (this._timeoutId) {
            GLib.source_remove(this._timeoutId);
            this._timeoutId = null;
        }
        if (this._sig) {
            this.settings.disconnect(this._sig);
            this._sig = null;
        }
        this.actor.destroy();
    }
}
