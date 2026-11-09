#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Purpose: Compiles GSettings schemas and installs the Kepler Shell extension locally.

set -e

UUID="kepler-shell@antonio.local"
EXTENSION_DIR="${HOME}/.local/share/gnome-shell/extensions/${UUID}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> Compiling GSettings schema..."
glib-compile-schemas "${SCRIPT_DIR}/schemas"

echo "==> Installing Kepler Shell Extension to ${EXTENSION_DIR}..."
mkdir -p "${EXTENSION_DIR}"
cp -r "${SCRIPT_DIR}"/* "${EXTENSION_DIR}/"

echo "==> Successfully installed Kepler Shell Extension!"
echo "To enable the extension, run:"
echo "    gnome-extensions enable ${UUID}"
echo "To open settings, run:"
echo "    gnome-extensions prefs ${UUID}"
