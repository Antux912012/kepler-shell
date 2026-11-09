// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Manages the Windows 11 style window tiling snap layouts overlay.
 * Architecture Position: Instantiated by the KeplerTaskbar component in taskbar.js.
 * Concurrency Constraints: Executes on the GNOME Shell main loop thread (GJS / Clutter).
 */

import Clutter from 'gi://Clutter';
import St from 'gi://St';
import Meta from 'gi://Meta';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

/**
 * Provides a popover overlay to snap active windows to predefined screen zones.
 */
export class SnapLayoutsMenu {
    /**
     * Creates and initializes the snap layouts overlay.
     */
    constructor() {
        this.actor = new St.BoxLayout({
            style_class: 'kepler-flyout-window kepler-snap-overlay',
            vertical: true,
            reactive: true,
            visible: false,
        });

        const title = new St.Label({
            text: 'Snap Window Layout',
            style: 'font-weight: bold; margin-bottom: 8px; color: #eceff4;',
        });
        this.actor.add_child(title);

        const gridBox = new St.BoxLayout({ vertical: true });

        // Row 1: Left and Right (50/50 Split)
        const row1 = new St.BoxLayout({ vertical: false });
        row1.add_child(this._createButton('Left 1/2', 'left', 80, 50));
        row1.add_child(this._createButton('Right 1/2', 'right', 80, 50));
        gridBox.add_child(row1);

        // Row 2: 2x2 Screen Quadrants
        const row2 = new St.BoxLayout({ vertical: false });
        row2.add_child(this._createButton('TL', 'top-left', 38, 38));
        row2.add_child(this._createButton('TR', 'top-right', 38, 38));
        row2.add_child(this._createButton('BL', 'bottom-left', 38, 38));
        row2.add_child(this._createButton('BR', 'bottom-right', 38, 38));
        gridBox.add_child(row2);

        // Row 3: Full Maximize
        const row3 = new St.BoxLayout({ vertical: false });
        const maxBtn = this._createButton('Full Maximize', 'maximize', 164, 34);
        row3.add_child(maxBtn);
        gridBox.add_child(row3);

        this.actor.add_child(gridBox);
        Main.uiGroup.add_child(this.actor);
    }

    /**
     * Creates a snap zone selection button.
     * @param {string} label - Display label text.
     * @param {string} position - Snap zone identifier.
     * @param {number} width - Button width in pixels.
     * @param {number} height - Button height in pixels.
     * @returns {St.Button} The initialized button actor.
     */
    _createButton(label, position, width, height) {
        const btn = new St.Button({
            label,
            style_class: 'kepler-snap-tile',
            can_focus: true,
            reactive: true,
            width,
            height,
        });
        btn.connect('clicked', () => {
            this.snapActiveWindow(position);
            this.hide();
        });
        return btn;
    }

    /**
     * Resizes and positions the focused Mutter window to the target zone.
     * @param {string} position - Snap zone identifier ('left', 'right', 'top-left', etc.).
     */
    snapActiveWindow(position) {
        const win = global.display.focus_window;
        if (!win) return;

        const monitor = win.get_monitor();
        const workArea = Main.layoutManager.getWorkAreaForMonitor(monitor);
        const { x, y, width, height } = workArea;
        const halfWidth = Math.floor(width / 2);
        const halfHeight = Math.floor(height / 2);

        if (win.maximized_horizontally || win.maximized_vertically) {
            win.unmaximize(Meta.MaximizeFlags.BOTH);
        }

        switch (position) {
            case 'left':
                win.move_resize_frame(true, x, y, halfWidth, height);
                break;
            case 'right':
                win.move_resize_frame(true, x + halfWidth, y, halfWidth, height);
                break;
            case 'top-left':
                win.move_resize_frame(true, x, y, halfWidth, halfHeight);
                break;
            case 'top-right':
                win.move_resize_frame(true, x + halfWidth, y, halfWidth, halfHeight);
                break;
            case 'bottom-left':
                win.move_resize_frame(true, x, y + halfHeight, halfWidth, halfHeight);
                break;
            case 'bottom-right':
                win.move_resize_frame(true, x + halfWidth, y + halfHeight, halfWidth, halfHeight);
                break;
            case 'maximize':
                win.maximize(Meta.MaximizeFlags.BOTH);
                break;
        }
    }

    /**
     * Toggles visibility of the snap layout overlay near the source button.
     * @param {Clutter.Actor} sourceActor - The button actor triggering this overlay.
     * @param {boolean} isTop - True if the taskbar docks at the top edge.
     */
    toggleAt(sourceActor, isTop) {
        if (this.actor.visible) {
            this.hide();
            return;
        }

        const [sourceX, sourceY] = sourceActor.get_transformed_position();
        const [, sourceHeight] = sourceActor.get_size();

        const posX = Math.max(10, sourceX - 60);
        const posY = isTop ? sourceY + sourceHeight + 8 : sourceY - 180;

        this.actor.set_position(posX, posY);
        this.actor.visible = true;
    }

    /**
     * Hides the snap layout overlay.
     */
    hide() {
        this.actor.visible = false;
    }

    /**
     * Destroys all child actors and disconnects event handlers.
     */
    destroy() {
        if (this.actor) {
            this.actor.destroy();
            this.actor = null;
        }
    }
}
