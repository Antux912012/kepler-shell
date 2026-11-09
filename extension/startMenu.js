// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Manages the Windows 11 style Start Menu flyout.
 * Architecture Position: Instantiated by KeplerShellExtension (extension.js), toggled by KeplerTaskbar.
 * Concurrency Constraints: Executes on the GNOME Shell main loop thread.
 */

import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';
import Shell from 'gi://Shell';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

/**
 * Provides the centered Start Menu with search, pinned tiles, recent files, and power options.
 */
export class StartMenuFlyout {
    /**
     * Initializes the Start Menu container and builds its internal widgets.
     * @param {Gio.Settings} settings - The Kepler Shell GSettings instance.
     */
    constructor(settings) {
        this.settings = settings;
        this.actor = new St.BoxLayout({
            style_class: 'kepler-flyout-window',
            vertical: true,
            reactive: true,
            visible: false,
        });

        this._appSys = Shell.AppSystem.get_default();
        this._buildUI();
        Main.uiGroup.add_child(this.actor);
    }

    /**
     * Constructs the visual layout of the Start Menu.
     */
    _buildUI() {
        this.actor.destroy_all_children();
        const width = this.settings.get_int('start-menu-width');
        const height = this.settings.get_int('start-menu-height');
        this.actor.set_size(width, height);

        // Search Bar and Calculator Badge
        const searchBox = new St.BoxLayout({ vertical: false, style: 'margin-bottom: 12px;' });
        this._searchEntry = new St.Entry({
            hint_text: 'Type here to search...',
            style_class: 'kepler-search-entry',
            can_focus: true,
            x_expand: true,
        });
        this._calcLabel = new St.Label({
            style_class: 'kepler-calc-badge',
            y_align: Clutter.ActorAlign.CENTER,
            visible: false,
        });

        this._searchEntry.clutter_text.connect('text-changed', () => this._onSearchChanged());
        searchBox.add_child(this._searchEntry);
        searchBox.add_child(this._calcLabel);
        this.actor.add_child(searchBox);

        // Pinned Section Header
        const pinnedHeader = new St.Label({
            text: 'Pinned',
            style: 'font-weight: bold; color: #d8dee9; margin-bottom: 8px;',
        });
        this.actor.add_child(pinnedHeader);

        this._pinnedGrid = new St.Widget({
            layout_manager: new Clutter.GridLayout({ orientation: Clutter.Orientation.HORIZONTAL }),
            style: 'margin-bottom: 16px;',
        });
        this._populatePinned();
        this.actor.add_child(this._pinnedGrid);

        // Recommended Section
        if (this.settings.get_boolean('show-recommended-files')) {
            const recHeader = new St.Label({
                text: 'Recommended',
                style: 'font-weight: bold; color: #d8dee9; margin-bottom: 6px;',
            });
            this.actor.add_child(recHeader);

            this._recBox = new St.BoxLayout({ vertical: true, y_expand: true });
            this._populateRecommended();
            this.actor.add_child(this._recBox);
        }

        // Power Action Footer
        this._buildPowerFooter();
    }

    /**
     * Fills the pinned applications grid.
     */
    _populatePinned() {
        const pinnedIds = this.settings.get_strv('pinned-apps');
        const layout = this._pinnedGrid.layout_manager;
        let col = 0;
        let row = 0;

        for (const appId of pinnedIds) {
            const app = this._appSys.lookup_app(appId);
            if (!app) continue;

            const btn = new St.Button({
                style_class: 'kepler-pinned-tile',
                reactive: true,
                can_focus: true,
            });

            const vbox = new St.BoxLayout({
                vertical: true,
                x_align: Clutter.ActorAlign.CENTER,
            });
            const icon = app.create_icon_texture(36);
            const lbl = new St.Label({
                text: app.get_name(),
                style_class: 'kepler-pinned-label',
            });

            vbox.add_child(icon);
            vbox.add_child(lbl);
            btn.set_child(vbox);

            btn.connect('clicked', () => {
                app.open_new_window(-1);
                this.hide();
            });

            layout.attach(btn, col, row, 1, 1);
            col++;
            if (col >= 5) {
                col = 0;
                row++;
            }
        }
    }

    /**
     * Populates recent files using the GTK RecentManager service.
     */
    _populateRecommended() {
        const maxItems = this.settings.get_int('max-recommended-files');
        const manager = Gtk.RecentManager.get_default();
        const items = manager.get_items();
        let count = 0;

        for (const info of items) {
            if (count >= maxItems) break;
            if (!info.exists()) continue;

            const btn = new St.Button({
                style_class: 'kepler-rec-item',
                reactive: true,
                x_expand: true,
            });
            const box = new St.BoxLayout({ vertical: false });
            const icon = new St.Icon({
                icon_name: 'text-x-generic',
                icon_size: 20,
                style: 'margin-right: 10px;',
            });
            const textVBox = new St.BoxLayout({ vertical: true });
            const title = new St.Label({
                text: info.get_display_name(),
                style_class: 'kepler-rec-title',
            });
            const uri = new St.Label({
                text: info.get_uri(),
                style_class: 'kepler-rec-subtitle',
            });

            textVBox.add_child(title);
            textVBox.add_child(uri);
            box.add_child(icon);
            box.add_child(textVBox);
            btn.set_child(box);

            const fileUri = info.get_uri();
            btn.connect('clicked', () => {
                Gio.AppInfo.launch_default_for_uri(fileUri, null);
                this.hide();
            });

            this._recBox.add_child(btn);
            count++;
        }
    }

    /**
     * Builds the bottom profile and system power action bar.
     */
    _buildPowerFooter() {
        const footer = new St.BoxLayout({
            style_class: 'kepler-power-footer',
            vertical: false,
            y_align: Clutter.ActorAlign.END,
        });

        const userLabel = new St.Label({
            text: GLib.get_user_name(),
            style: 'font-weight: bold; color: #eceff4;',
            x_expand: true,
            y_align: Clutter.ActorAlign.CENTER,
        });
        footer.add_child(userLabel);

        const lockBtn = this._createPowerBtn('system-lock-screen-symbolic', ['loginctl', 'lock-session']);
        const restartBtn = this._createPowerBtn('system-reboot-symbolic', ['systemctl', 'reboot']);
        const shutdownBtn = this._createPowerBtn('system-shutdown-symbolic', ['systemctl', 'poweroff']);

        footer.add_child(lockBtn);
        footer.add_child(restartBtn);
        footer.add_child(shutdownBtn);
        this.actor.add_child(footer);
    }

    /**
     * Creates an individual system control power button using direct process execution.
     * @param {string} iconName - FreeDesktop icon name.
     * @param {string[]} argv - Argument array for execution.
     * @returns {St.Button} The power button actor.
     */
    _createPowerBtn(iconName, argv) {
        const btn = new St.Button({
            style_class: 'kepler-power-btn',
            reactive: true,
            can_focus: true,
        });
        btn.set_child(new St.Icon({ icon_name: iconName, icon_size: 18 }));
        btn.connect('clicked', () => {
            this.hide();
            GLib.spawn_async(null, argv, null, GLib.SpawnFlags.SEARCH_PATH, null);
        });
        return btn;
    }

    /**
     * Evaluates search box input for mathematical expressions.
     */
    _onSearchChanged() {
        const text = this._searchEntry.get_text().trim();
        if (this.settings.get_boolean('enable-calculator')) {
            const mathResult = this._evaluateArithmetic(text);
            if (mathResult !== null) {
                this._calcLabel.text = `= ${mathResult}`;
                this._calcLabel.visible = true;
                return;
            }
        }
        this._calcLabel.visible = false;
    }

    /**
     * Deterministic, safe mathematical evaluator without eval.
     * @param {string} expression - The arithmetic expression.
     * @returns {number|null} The numerical result or null if invalid.
     */
    _evaluateArithmetic(expression) {
        const clean = expression.replace(/\s+/g, '');
        if (clean.length < 3) return null;

        // Support binary operations: +, -, *, /
        const operators = ['+', '-', '*', '/'];
        for (const op of operators) {
            const index = clean.lastIndexOf(op);
            if (index > 0 && index < clean.length - 1) {
                const leftStr = clean.slice(0, index);
                const rightStr = clean.slice(index + 1);
                const left = Number(leftStr);
                const right = Number(rightStr);

                if (!isNaN(left) && !isNaN(right)) {
                    switch (op) {
                        case '+': return left + right;
                        case '-': return left - right;
                        case '*': return left * right;
                        case '/': return right !== 0 ? Math.round((left / right) * 1000) / 1000 : null;
                    }
                }
            }
        }
        return null;
    }

    /**
     * Toggles visibility of the Start Menu near the taskbar start button.
     * @param {Clutter.Actor} sourceActor - Source button triggering this menu.
     * @param {boolean} isTop - True if the taskbar docks at the top edge.
     * @param {string} alignment - Alignment mode ('center' or 'left').
     */
    toggleAt(sourceActor, isTop, alignment) {
        if (this.actor.visible) {
            this.hide();
            return;
        }

        const margin = this.settings.get_int('flyout-margin');
        const [menuWidth, menuHeight] = this.actor.get_size();
        const primary = Main.layoutManager.primaryMonitor;
        const posX = alignment === 'left' ? primary.x + 16 : primary.x + Math.floor((primary.width - menuWidth) / 2);
        const posY = isTop ? primary.y + 48 + margin : primary.y + primary.height - 48 - menuHeight - margin;

        this.actor.set_position(posX, posY);
        this.actor.visible = true;
        this._searchEntry.grab_key_focus();
    }

    /**
     * Hides the Start Menu flyout and clears the search box.
     */
    hide() {
        this.actor.visible = false;
        if (this._searchEntry) {
            this._searchEntry.set_text('');
        }
    }

    /**
     * Destroys child actors and clears resources.
     */
    destroy() {
        if (this.actor) {
            this.actor.destroy();
            this.actor = null;
        }
    }
}
