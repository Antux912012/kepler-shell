// Kepler Shell Mutter Bridge Extension for GNOME Shell 45+
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const IFACE_XML = `
<node>
  <interface name="org.gnome.Shell.Extensions.KeplerBridge">
    <method name="GetWindows">
      <arg type="a(sssubb)" direction="out" name="windows"/>
    </method>
    <method name="FocusWindow">
      <arg type="s" direction="in" name="id"/>
    </method>
    <method name="MinimizeWindow">
      <arg type="s" direction="in" name="id"/>
    </method>
    <method name="CloseWindow">
      <arg type="s" direction="in" name="id"/>
    </method>
    <method name="SnapWindow">
      <arg type="s" direction="in" name="id"/>
      <arg type="s" direction="in" name="position"/>
    </method>
    <method name="GetWorkspaces">
      <arg type="u" direction="out" name="active"/>
      <arg type="u" direction="out" name="count"/>
    </method>
    <method name="SwitchWorkspace">
      <arg type="u" direction="in" name="index"/>
    </method>
    <signal name="WindowsChanged"/>
    <signal name="ActiveWindowChanged">
      <arg type="s" name="id"/>
    </signal>
    <signal name="WorkspaceChanged">
      <arg type="u" name="active"/>
    </signal>
  </interface>
</node>`;

export default class KeplerBridgeExtension extends Extension {
    enable() {
        this._nodeInfo = Gio.DBusNodeInfo.new_for_xml(IFACE_XML);
        this._ifaceInfo = this._nodeInfo.interfaces[0];
        this._signals = [];

        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(this._ifaceInfo, this);
        this._dbusOwnerId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            'org.gnome.Shell.Extensions.KeplerBridge',
            Gio.BusNameOwnerFlags.REPLACE,
            (connection) => {
                this._dbusConnection = connection;
                this._dbusImpl.export(connection, '/org/gnome/Shell/Extensions/KeplerBridge');
            },
            null,
            () => console.warn('[KeplerBridge] Lost D-Bus name ownership')
        );

        this._connectSignals();
        console.log('[KeplerBridge] Extension enabled successfully');
    }

    disable() {
        if (this._dbusOwnerId) {
            Gio.bus_unown_name(this._dbusOwnerId);
            this._dbusOwnerId = null;
        }
        if (this._dbusImpl && this._dbusConnection) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }
        this._disconnectSignals();
        console.log('[KeplerBridge] Extension disabled');
    }

    _connectSignals() {
        const display = global.display;
        this._signals.push(
            display.connect('window-created', () => this._emitWindowsChanged()),
            display.connect('notify::focus-window', () => this._emitFocusChanged())
        );

        const wsManager = global.workspace_manager;
        this._signals.push(
            wsManager.connect('active-workspace-changed', () => this._emitWorkspaceChanged()),
            wsManager.connect('workspace-added', () => this._emitWorkspaceChanged()),
            wsManager.connect('workspace-removed', () => this._emitWorkspaceChanged())
        );
    }

    _disconnectSignals() {
        const display = global.display;
        const wsManager = global.workspace_manager;
        for (const id of this._signals) {
            try {
                display.disconnect(id);
                wsManager.disconnect(id);
            } catch (_) {}
        }
        this._signals = [];
    }

    _emitWindowsChanged() {
        if (!this._dbusConnection) return;
        this._dbusConnection.emit_signal(
            null,
            '/org/gnome/Shell/Extensions/KeplerBridge',
            'org.gnome.Shell.Extensions.KeplerBridge',
            'WindowsChanged',
            null
        );
    }

    _emitFocusChanged() {
        if (!this._dbusConnection) return;
        const win = global.display.focus_window;
        const id = win ? String(win.get_id()) : "";
        this._dbusConnection.emit_signal(
            null,
            '/org/gnome/Shell/Extensions/KeplerBridge',
            'org.gnome.Shell.Extensions.KeplerBridge',
            'ActiveWindowChanged',
            new GLib.Variant('(s)', [id])
        );
    }

    _emitWorkspaceChanged() {
        if (!this._dbusConnection) return;
        const ws = global.workspace_manager.get_active_workspace_index();
        this._dbusConnection.emit_signal(
            null,
            '/org/gnome/Shell/Extensions/KeplerBridge',
            'org.gnome.Shell.Extensions.KeplerBridge',
            'WorkspaceChanged',
            new GLib.Variant('(u)', [ws])
        );
    }

    _findWindowById(id) {
        const targetId = parseInt(id, 10);
        const windows = global.get_window_actors().map(a => a.meta_window).filter(Boolean);
        return windows.find(w => w.get_id() === targetId);
    }

    GetWindows() {
        const windows = [];
        const actors = global.get_window_actors();
        const focusWin = global.display.focus_window;

        for (const actor of actors) {
            const win = actor.meta_window;
            if (!win || win.is_skip_taskbar()) continue;

            const id = String(win.get_id());
            const title = win.get_title() || "";
            const wmClass = win.get_wm_class() || "";
            const ws = win.get_workspace() ? win.get_workspace().index() : 0;
            const active = focusWin ? (win.get_id() === focusWin.get_id()) : false;
            const minimized = win.minimized;

            windows.push([id, title, wmClass, ws, active, minimized]);
        }
        return [windows];
    }

    FocusWindow(id) {
        const win = this._findWindowById(id);
        if (win) {
            if (win.minimized) win.unminimize();
            win.activate(global.get_current_time());
        }
    }

    MinimizeWindow(id) {
        const win = this._findWindowById(id);
        if (win && !win.minimized) {
            win.minimize();
        }
    }

    CloseWindow(id) {
        const win = this._findWindowById(id);
        if (win) {
            win.delete(global.get_current_time());
        }
    }

    SnapWindow(id, position) {
        const win = this._findWindowById(id);
        if (!win) return;

        const monitor = win.get_monitor();
        const workArea = Main.layoutManager.getWorkAreaForMonitor(monitor);
        const { x, y, width, height } = workArea;
        const halfW = Math.floor(width / 2);
        const halfH = Math.floor(height / 2);

        if (win.maximized_horizontally || win.maximized_vertically) {
            win.unmaximize(Meta.MaximizeFlags.BOTH);
        }

        switch (position) {
            case 'left':
                win.move_resize_frame(true, x, y, halfW, height);
                break;
            case 'right':
                win.move_resize_frame(true, x + halfW, y, halfW, height);
                break;
            case 'top-left':
                win.move_resize_frame(true, x, y, halfW, halfH);
                break;
            case 'top-right':
                win.move_resize_frame(true, x + halfW, y, halfW, halfH);
                break;
            case 'bottom-left':
                win.move_resize_frame(true, x, y + halfH, halfW, halfH);
                break;
            case 'bottom-right':
                win.move_resize_frame(true, x + halfW, y + halfH, halfW, halfH);
                break;
            case 'maximize':
                win.maximize(Meta.MaximizeFlags.BOTH);
                break;
        }
    }

    GetWorkspaces() {
        const wsManager = global.workspace_manager;
        return [wsManager.get_active_workspace_index(), wsManager.get_n_workspaces()];
    }

    SwitchWorkspace(index) {
        const wsManager = global.workspace_manager;
        const ws = wsManager.get_workspace_by_index(index);
        if (ws) {
            ws.activate(global.get_current_time());
        }
    }
}
