// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Serves as the primary entry point and lifecycle manager for the Kepler Shell GNOME extension.
 * Architecture Position: Instantiated directly by GNOME Shell extension subsystem.
 * Concurrency Constraints: Executes exclusively on the GNOME Shell main loop thread.
 */

import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { KeplerTaskbar } from './taskbar.js';
import { StartMenuFlyout } from './startMenu.js';
import { ControlCenterFlyout } from './controlCenter.js';
import { SnapLayoutsMenu } from './snapLayouts.js';

/**
 * Manages initialization, component activation, and state teardown for Kepler Shell.
 */
export default class KeplerShellExtension extends Extension {
    /**
     * Activates the extension components when enabled by GNOME Shell.
     */
    enable() {
        this._settings = this.getSettings();

        // 1. Instantiate Flyouts and Popovers
        this._snapMenu = new SnapLayoutsMenu();
        this._startMenu = new StartMenuFlyout(this._settings);
        this._controlCenter = new ControlCenterFlyout(this._settings);

        // 2. Instantiate Main Taskbar
        this._taskbar = new KeplerTaskbar(
            this._settings,
            this._startMenu,
            this._controlCenter,
            this._snapMenu
        );

        // 3. Hide Default GNOME Top Panel when Bottom Taskbar is Selected
        if (this._settings.get_string('taskbar-position') === 'bottom') {
            Main.panel.opacity = 0;
            Main.panel.reactive = false;
        }

        this._panelSig = this._settings.connect('changed::taskbar-position', () => {
            const isBottom = this._settings.get_string('taskbar-position') === 'bottom';
            Main.panel.opacity = isBottom ? 0 : 255;
            Main.panel.reactive = !isBottom;
        });

        console.log('[KeplerShell] GNOME Shell Extension enabled successfully');
    }

    /**
     * Deactivates all components and restores default GNOME Shell state.
     */
    disable() {
        if (this._panelSig) {
            this._settings.disconnect(this._panelSig);
            this._panelSig = null;
        }

        // Restore Default GNOME Shell Panel
        Main.panel.opacity = 255;
        Main.panel.reactive = true;

        if (this._taskbar) {
            this._taskbar.destroy();
            this._taskbar = null;
        }
        if (this._startMenu) {
            this._startMenu.destroy();
            this._startMenu = null;
        }
        if (this._controlCenter) {
            this._controlCenter.destroy();
            this._controlCenter = null;
        }
        if (this._snapMenu) {
            this._snapMenu.destroy();
            this._snapMenu = null;
        }

        this._settings = null;
        console.log('[KeplerShell] GNOME Shell Extension disabled');
    }
}
