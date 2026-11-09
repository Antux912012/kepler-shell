// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Provides the Windows 11 style Quick Settings / Control Center flyout.
 * Architecture Position: Instantiated by KeplerShellExtension (extension.js), toggled by SystrayPill.
 * Concurrency Constraints: Executes on the GNOME Shell main loop thread.
 */

import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

/**
 * Renders quick system toggle buttons and hardware sliders.
 */
export class ControlCenterFlyout {
    /**
     * Initializes the Control Center flyout container and child controls.
     * @param {Gio.Settings} settings - The Kepler Shell GSettings instance.
     */
    constructor(settings) {
        this.settings = settings;
        this.actor = new St.BoxLayout({
            style_class: 'kepler-flyout-window',
            vertical: true,
            reactive: true,
            visible: false,
            width: 360,
        });

        this._buildUI();
        Main.uiGroup.add_child(this.actor);
    }

    /**
     * Constructs the Quick Settings grid and slider controls.
     */
    _buildUI() {
        this.actor.destroy_all_children();

        const title = new St.Label({
            text: 'Quick Settings',
            style: 'font-weight: bold; color: #eceff4; margin-bottom: 12px;',
        });
        this.actor.add_child(title);

        // 2x2 Grid of Quick Toggle Pills
        const grid = new St.Widget({
            layout_manager: new Clutter.GridLayout({ orientation: Clutter.Orientation.HORIZONTAL }),
            style: 'margin-bottom: 14px;',
        });
        const layout = grid.layout_manager;

        const wifiToggle = this._createToggle('network-wireless-symbolic', 'Wi-Fi', true, (active) => {
            const stateArg = active ? 'on' : 'off';
            GLib.spawn_async(null, ['nmcli', 'radio', 'wifi', stateArg], null, GLib.SpawnFlags.SEARCH_PATH, null);
        });

        const btToggle = this._createToggle('bluetooth-active-symbolic', 'Bluetooth', true, (active) => {
            const action = active ? 'unblock' : 'block';
            GLib.spawn_async(null, ['rfkill', action, 'bluetooth'], null, GLib.SpawnFlags.SEARCH_PATH, null);
        });

        const nlToggle = this._createToggle('night-light-symbolic', 'Night Light', false, (active) => {
            const boolStr = active ? 'true' : 'false';
            GLib.spawn_async(null, ['gsettings', 'set', 'org.gnome.settings-daemon.plugins.color', 'night-light-enabled', boolStr], null, GLib.SpawnFlags.SEARCH_PATH, null);
        });

        const dndToggle = this._createToggle('notifications-disabled-symbolic', 'Do Not Disturb', false, (active) => {
            const bannerStr = active ? 'false' : 'true';
            GLib.spawn_async(null, ['gsettings', 'set', 'org.gnome.desktop.notifications', 'show-banners', bannerStr], null, GLib.SpawnFlags.SEARCH_PATH, null);
        });

        layout.attach(wifiToggle, 0, 0, 1, 1);
        layout.attach(btToggle, 1, 0, 1, 1);
        layout.attach(nlToggle, 0, 1, 1, 1);
        layout.attach(dndToggle, 1, 1, 1, 1);
        this.actor.add_child(grid);

        // Volume Adjustment Row
        this.actor.add_child(this._createSliderRow('audio-volume-high-symbolic', 65, (value) => {
            const fraction = (value / 100).toFixed(2);
            GLib.spawn_async(null, ['wpctl', 'set-volume', '@DEFAULT_AUDIO_SINK@', fraction], null, GLib.SpawnFlags.SEARCH_PATH, null);
        }));

        // Brightness Adjustment Row
        this.actor.add_child(this._createSliderRow('display-brightness-symbolic', 80, (value) => {
            const percentStr = `${Math.round(value)}%`;
            GLib.spawn_async(null, ['brightnessctl', 'set', percentStr], null, GLib.SpawnFlags.SEARCH_PATH, null);
        }));
    }

    /**
     * Creates an individual quick toggle button actor.
     * @param {string} iconName - FreeDesktop icon name.
     * @param {string} labelText - Display text.
     * @param {boolean} defaultActive - Initial toggle state.
     * @param {Function} callback - Callback function receiving the new boolean state.
     * @returns {St.Button} The toggle button actor.
     */
    _createToggle(iconName, labelText, defaultActive, callback) {
        let active = defaultActive;
        const btn = new St.Button({
            style_class: active ? 'kepler-quick-toggle active' : 'kepler-quick-toggle',
            reactive: true,
            can_focus: true,
            x_expand: true,
        });

        const box = new St.BoxLayout({ vertical: false, y_align: Clutter.ActorAlign.CENTER });
        const icon = new St.Icon({ icon_name: iconName, icon_size: 16, style: 'margin-right: 8px;' });
        const label = new St.Label({ text: labelText, style: 'font-weight: 500;' });
        box.add_child(icon);
        box.add_child(label);
        btn.set_child(box);

        btn.connect('clicked', () => {
            active = !active;
            if (active) {
                btn.add_style_class_name('active');
            } else {
                btn.remove_style_class_name('active');
            }
            callback(active);
        });

        return btn;
    }

    /**
     * Creates a slider control row with an icon and a drawing area.
     * @param {string} iconName - FreeDesktop icon name.
     * @param {number} initialValue - Initial value (0 to 100).
     * @param {Function} callback - Callback receiving the updated numerical value.
     * @returns {St.BoxLayout} The slider row container.
     */
    _createSliderRow(iconName, initialValue, callback) {
        const row = new St.BoxLayout({
            vertical: false,
            style: 'margin-top: 8px;',
            y_align: Clutter.ActorAlign.CENTER,
        });
        const icon = new St.Icon({ icon_name: iconName, icon_size: 18, style: 'margin-right: 12px;' });
        const slider = new St.DrawingArea({
            style: 'background-color: rgba(255, 255, 255, 0.15); border-radius: 4px; height: 6px;',
            x_expand: true,
            reactive: true,
        });

        slider.connect('button-press-event', (_, event) => {
            const [clickedX] = event.get_coords();
            const [originX] = slider.get_transformed_position();
            const [sliderWidth] = slider.get_size();
            const fraction = Math.max(0.0, Math.min(1.0, (clickedX - originX) / sliderWidth));
            callback(fraction * 100.0);
            return Clutter.EVENT_STOP;
        });

        row.add_child(icon);
        row.add_child(slider);
        return row;
    }

    /**
     * Toggles visibility of the Control Center flyout near the status pill.
     * @param {Clutter.Actor} sourceActor - Source button triggering this flyout.
     * @param {boolean} isTop - True if the taskbar docks at the top edge.
     */
    toggleAt(sourceActor, isTop) {
        if (this.actor.visible) {
            this.hide();
            return;
        }

        const margin = this.settings.get_int('flyout-margin');
        const primary = Main.layoutManager.primaryMonitor;
        const posX = primary.x + primary.width - 360 - 16;
        const posY = isTop ? primary.y + 48 + margin : primary.y + primary.height - 48 - 280 - margin;

        this.actor.set_position(posX, posY);
        this.actor.visible = true;
    }

    /**
     * Hides the Control Center flyout.
     */
    hide() {
        this.actor.visible = false;
    }

    /**
     * Destroys child actors and releases resources.
     */
    destroy() {
        if (this.actor) {
            this.actor.destroy();
            this.actor = null;
        }
    }
}
