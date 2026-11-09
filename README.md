<!--
SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Kepler Shell

Kepler Shell is a desktop user interface for Linux systems running GNOME Shell. It provides a taskbar, centered application dock, Start Menu flyout, Quick Settings panel, and window snap layout overlays inspired by modern desktop interfaces.

## Key Features

- **Bottom Taskbar**: Configurable 48px panel located at the bottom or top of the display.
- **Application Dock**: Displays pinned application shortcuts and running applications with Windows 11 style active indicator pills.
- **Start Menu Flyout**: Centered flyout window with application search, arithmetic calculation parser, pinned applications grid, recent file recommendations, and system power actions.
- **Control Center**: Quick toggle buttons for Wi-Fi, Bluetooth, Night Light, and Do Not Disturb, combined with volume and brightness sliders.
- **Snap Layouts**: Window tiling menu providing 50/50 half split, 2x2 quadrants, and full maximize presets.
- **Libadwaita Preferences**: Complete graphical configuration dialog to personalize layout, dimensions, themes, and feature toggles.

## System Architecture

```text
+-------------------------------------------------------------------+
|                        GNOME Shell Process                        |
|                                                                   |
|   +-----------------------------------------------------------+   |
|   |                  KeplerShellExtension                     |   |
|   |                     (extension.js)                        |   |
|   +-----------------------------+-----------------------------+   |
|                                 |                                 |
|         +-----------------------+-----------------------+         |
|         |                       |                       |         |
|   +-----+-----+           +-----+-----+           +-----+-----+   |
|   |  Taskbar  |           | StartMenu |           |  Control  |   |
|   | (taskbar) |           |(startMenu)|           |  Center   |   |
|   +-----+-----+           +-----------+           +-----------+   |
|         |                                                         |
|   +-----+-----+                                                   |
|   |  AppDock  |                                                   |
|   | (appDock) |                                                   |
|   +-----------+                                                   |
+---------------------------------+---------------------------------+
                                  |
               +------------------+------------------+
               | GSettings Engine (glib-schemas)    |
               | Path: /org/gnome/shell/extensions/  |
               +-------------------------------------+
```

## Prerequisites and Dependencies

### Required System Software:
- GNOME Shell version 45, 46, 47, 48, 49, or 50.
- GJS (GNOME JavaScript Engine).
- Libadwaita and GTK4.
- `glib-compile-schemas` (part of `libglib2.0-bin` or `glib2-devel`).

### Supported Linux Distributions:
- Fedora 40, 41, 42, 43, 44.
- Debian 12 (Bookworm), 13 (Trixie), or Sid.
- Ubuntu 24.04 LTS (Noble Numbat) or newer.
- Arch Linux.

## Installation Instructions

1. Clone or copy the repository to your local storage:
   ```bash
   cd /run/media/antonio/DevDisk/Projects/kepler-shell/extension
   ```

2. Run the installation script:
   ```bash
   ./install.sh
   ```

3. Restart GNOME Shell:
   - On **X11**: Press `Alt + F2`, type `r`, and press `Enter`.
   - On **Wayland**: Log out and log back in to your user session.

4. Enable the extension:
   ```bash
   gnome-extensions enable kepler-shell@antonio.local
   ```

5. Open the configuration dialog:
   ```bash
   gnome-extensions prefs kepler-shell@antonio.local
   ```

## Configuration Settings

All options store values inside the GSettings path `/org/gnome/shell/extensions/kepler-shell/`:

| Schema Key | Data Type | Default Value | Description |
|---|---|---|---|
| `taskbar-position` | string | `'bottom'` | Screen edge for the taskbar (`'bottom'` or `'top'`). |
| `taskbar-alignment` | string | `'center'` | Alignment of application dock icons (`'center'` or `'left'`). |
| `taskbar-height` | integer | `48` | Height of the taskbar in pixels (36 to 64). |
| `icon-size` | integer | `28` | Dimension of dock application icons in pixels (20 to 48). |
| `theme-palette` | string | `'nord'` | Color theme palette name (`'nord'`, `'win11-blue'`, `'mica-dark'`). |
| `acrylic-opacity` | double | `0.82` | Opacity fraction for acrylic surfaces (0.30 to 1.00). |
| `corner-radius` | integer | `12` | Border radius for flyouts in pixels (0 to 24). |
| `flyout-margin` | integer | `8` | Distance between the taskbar and flyouts in pixels. |
| `start-menu-width` | integer | `640` | Width of the Start Menu in pixels. |
| `start-menu-height` | integer | `680` | Height of the Start Menu in pixels. |
| `enable-calculator` | boolean | `true` | Evaluates arithmetic expressions in the search input box. |
| `enable-snap-layouts` | boolean | `true` | Enables the window tiling snap overlay menu. |

## Privacy and Regulatory Compliance

- **GDPR Compliance**: Kepler Shell collects zero personal telemetry, stores no network tracking identifiers, and makes no outbound network connections. All configuration remains strictly local to the machine.
- **Cyber Resilience Act (CRA)**: Designed under secure-by-default principles. External processes execute through explicit argument arrays without shell interpolation.

## License

This project is licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later). See the LICENSE file for legal terms.
