// SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * File Responsibility: Provides the Libadwaita settings dialog for Kepler Shell.
 * Architecture Position: Instantiated by the GNOME Extensions app through the ExtensionPreferences class.
 * Concurrency Constraints: Executes exclusively on the GTK4 main loop thread.
 */

import { ExtensionPreferences, gettext as _ } from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';
import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

/**
 * Implements the preferences configuration provider for Kepler Shell.
 */
export default class KeplerShellPreferences extends ExtensionPreferences {
    /**
     * Builds and populates the Libadwaita preferences window pages.
     * @param {Adw.PreferencesWindow} window - The window container provided by the extensions app.
     */
    fillPreferencesWindow(window) {
        const settings = this.getSettings();

        // 1. Taskbar and Dock Page
        const taskbarPage = new Adw.PreferencesPage({
            title: _('Taskbar and Dock'),
            icon_name: 'user-desktop-symbolic',
        });

        const layoutGroup = new Adw.PreferencesGroup({
            title: _('Layout and Placement'),
            description: _('Configure the position and sizing of the Kepler taskbar.'),
        });

        // Position Selection
        const posRow = new Adw.ComboRow({
            title: _('Taskbar Position'),
            subtitle: _('Screen edge where the taskbar docks'),
            model: new Gtk.StringList({
                strings: [_('Bottom'), _('Top')],
            }),
        });
        const currentPos = settings.get_string('taskbar-position');
        posRow.selected = currentPos === 'top' ? 1 : 0;
        posRow.connect('notify::selected', () => {
            settings.set_string('taskbar-position', posRow.selected === 1 ? 'top' : 'bottom');
        });
        layoutGroup.add(posRow);

        // Alignment Selection
        const alignRow = new Adw.ComboRow({
            title: _('Icon Alignment'),
            subtitle: _('Align application icons in the center or to the left'),
            model: new Gtk.StringList({
                strings: [_('Center (Windows 11)'), _('Left (Classic)')],
            }),
        });
        const currentAlign = settings.get_string('taskbar-alignment');
        alignRow.selected = currentAlign === 'left' ? 1 : 0;
        alignRow.connect('notify::selected', () => {
            settings.set_string('taskbar-alignment', alignRow.selected === 1 ? 'left' : 'center');
        });
        layoutGroup.add(alignRow);

        // Taskbar Height
        const heightRow = new Adw.SpinRow({
            title: _('Taskbar Height (px)'),
            subtitle: _('Vertical height of the main taskbar in pixels'),
            adjustment: new Gtk.Adjustment({
                lower: 36,
                upper: 64,
                step_increment: 2,
                value: settings.get_int('taskbar-height'),
            }),
        });
        settings.bind('taskbar-height', heightRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        layoutGroup.add(heightRow);

        // Icon Size
        const iconSizeRow = new Adw.SpinRow({
            title: _('Application Icon Size (px)'),
            subtitle: _('Dimension of application dock icons in pixels'),
            adjustment: new Gtk.Adjustment({
                lower: 20,
                upper: 48,
                step_increment: 2,
                value: settings.get_int('icon-size'),
            }),
        });
        settings.bind('icon-size', iconSizeRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        layoutGroup.add(iconSizeRow);

        // Behavior Settings Group
        const behaviorGroup = new Adw.PreferencesGroup({
            title: _('Dock Behavior and Clock'),
        });

        const runningAppsRow = new Adw.SwitchRow({
            title: _('Show Running Applications'),
            subtitle: _('Display active applications on the dock with indicator pills'),
        });
        settings.bind('show-running-apps', runningAppsRow, 'active', Gio.SettingsBindFlags.DEFAULT);
        behaviorGroup.add(runningAppsRow);

        const clockRow = new Adw.EntryRow({
            title: _('Clock Format String'),
            text: settings.get_string('clock-format'),
        });
        clockRow.connect('changed', () => {
            settings.set_string('clock-format', clockRow.get_text() || '%H:%M\n%d/%m/%Y');
        });
        behaviorGroup.add(clockRow);

        taskbarPage.add(layoutGroup);
        taskbarPage.add(behaviorGroup);
        window.add(taskbarPage);

        // 2. Appearance and Theme Page
        const themePage = new Adw.PreferencesPage({
            title: _('Appearance'),
            icon_name: 'preferences-desktop-theme-symbolic',
        });

        const themeGroup = new Adw.PreferencesGroup({
            title: _('Color Theme and Acrylic Effect'),
            description: _('Control the Fluent visual materials and color themes.'),
        });

        const paletteRow = new Adw.ComboRow({
            title: _('Color Palette'),
            subtitle: _('Base color theme and accent highlighting'),
            model: new Gtk.StringList({
                strings: [_('Nord'), _('Windows 11 Blue'), _('Mica Dark'), _('Aurora Green'), _('Cyberpunk')],
            }),
        });
        const palettes = ['nord', 'win11-blue', 'mica-dark', 'aurora', 'cyberpunk'];
        const currentPalette = settings.get_string('theme-palette');
        const paletteIndex = palettes.indexOf(currentPalette);
        paletteRow.selected = paletteIndex >= 0 ? paletteIndex : 0;
        paletteRow.connect('notify::selected', () => {
            settings.set_string('theme-palette', palettes[paletteRow.selected]);
        });
        themeGroup.add(paletteRow);

        // Acrylic Opacity
        const opacityRow = new Adw.ActionRow({
            title: _('Acrylic Transparency Opacity'),
            subtitle: _('Lower values give higher translucency'),
        });
        const opacityScale = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 30, 100, 5);
        opacityScale.set_hexpand(true);
        opacityScale.set_size_request(160, -1);
        opacityScale.set_value(Math.round(settings.get_double('acrylic-opacity') * 100));
        opacityScale.connect('value-changed', () => {
            settings.set_double('acrylic-opacity', opacityScale.get_value() / 100.0);
        });
        opacityRow.add_suffix(opacityScale);
        themeGroup.add(opacityRow);

        // Corner Radius
        const radiusRow = new Adw.SpinRow({
            title: _('Corner Radius (px)'),
            subtitle: _('Border roundness for flyouts and taskbar elements'),
            adjustment: new Gtk.Adjustment({
                lower: 0,
                upper: 24,
                step_increment: 2,
                value: settings.get_int('corner-radius'),
            }),
        });
        settings.bind('corner-radius', radiusRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        themeGroup.add(radiusRow);

        // Flyout Margin
        const marginRow = new Adw.SpinRow({
            title: _('Flyout Detached Margin (px)'),
            subtitle: _('Distance separating flyouts from the taskbar edge'),
            adjustment: new Gtk.Adjustment({
                lower: 0,
                upper: 32,
                step_increment: 2,
                value: settings.get_int('flyout-margin'),
            }),
        });
        settings.bind('flyout-margin', marginRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        themeGroup.add(marginRow);

        themePage.add(themeGroup);
        window.add(themePage);

        // 3. Start Menu Page
        const startPage = new Adw.PreferencesPage({
            title: _('Start Menu'),
            icon_name: 'view-app-grid-symbolic',
        });

        const startDimGroup = new Adw.PreferencesGroup({
            title: _('Dimensions'),
            description: _('Custom sizing for the Windows 11 style Start Menu flyout.'),
        });

        const startWidthRow = new Adw.SpinRow({
            title: _('Start Menu Width (px)'),
            adjustment: new Gtk.Adjustment({
                lower: 480,
                upper: 840,
                step_increment: 20,
                value: settings.get_int('start-menu-width'),
            }),
        });
        settings.bind('start-menu-width', startWidthRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        startDimGroup.add(startWidthRow);

        const startHeightRow = new Adw.SpinRow({
            title: _('Start Menu Height (px)'),
            adjustment: new Gtk.Adjustment({
                lower: 480,
                upper: 880,
                step_increment: 20,
                value: settings.get_int('start-menu-height'),
            }),
        });
        settings.bind('start-menu-height', startHeightRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        startDimGroup.add(startHeightRow);

        const startFeaturesGroup = new Adw.PreferencesGroup({
            title: _('Search and Recommendations'),
        });

        const calcRow = new Adw.SwitchRow({
            title: _('Instant Math Calculator'),
            subtitle: _('Calculate arithmetic expressions directly inside the Start search box'),
        });
        settings.bind('enable-calculator', calcRow, 'active', Gio.SettingsBindFlags.DEFAULT);
        startFeaturesGroup.add(calcRow);

        const recRow = new Adw.SwitchRow({
            title: _('Show Recommended Files'),
            subtitle: _('Display recent documents below pinned applications'),
        });
        settings.bind('show-recommended-files', recRow, 'active', Gio.SettingsBindFlags.DEFAULT);
        startFeaturesGroup.add(recRow);

        const maxRecRow = new Adw.SpinRow({
            title: _('Maximum Recommended Files'),
            adjustment: new Gtk.Adjustment({
                lower: 2,
                upper: 12,
                step_increment: 1,
                value: settings.get_int('max-recommended-files'),
            }),
        });
        settings.bind('max-recommended-files', maxRecRow, 'value', Gio.SettingsBindFlags.DEFAULT);
        startFeaturesGroup.add(maxRecRow);

        startPage.add(startDimGroup);
        startPage.add(startFeaturesGroup);
        window.add(startPage);

        // 4. Window Management and Snap Layouts
        const windowPage = new Adw.PreferencesPage({
            title: _('Multitasking'),
            icon_name: 'view-grid-symbolic',
        });

        const snapGroup = new Adw.PreferencesGroup({
            title: _('Snap Layouts'),
            description: _('Windows 11 rapid tiling presets for window management.'),
        });

        const snapRow = new Adw.SwitchRow({
            title: _('Enable Snap Layouts Popup'),
            subtitle: _('Access snap presets from the taskbar and window overlays'),
        });
        settings.bind('enable-snap-layouts', snapRow, 'active', Gio.SettingsBindFlags.DEFAULT);
        snapGroup.add(snapRow);

        windowPage.add(snapGroup);
        window.add(windowPage);
    }
}
