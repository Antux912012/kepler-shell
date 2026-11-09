#![allow(dead_code)]

mod backend;
mod config;
mod ipc;
mod services;
mod ui;

use gtk4::prelude::*;
use libadwaita::Application;
use std::sync::Arc;
use std::time::Duration;

use backend::mutter_dbus::MutterDbusBackend;
use backend::wayland_layer::WaylandLayerBackend;
use backend::{CompositorBackend, SnapPosition};
use config::theme::ThemeManager;
use config::{load_config, start_config_watcher, Config};
use ipc::{IpcClient, IpcCommand, IpcServer};
use services::{
    AudioService, AudioState, MprisService, MprisState, NetworkService, NetworkState,
    NotificationItem, NotificationService, PowerService, PowerState, TrayEvent, TrayService,
};
use ui::{ControlCenterWindow, OsdWindow, StartMenuWindow, TaskbarWindow};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check if running as IPC client
    if args.len() > 1 && (args[1] == "msg" || args[1].starts_with("--")) {
        handle_ipc_client(&args[1..]);
        return;
    }

    // Initialize Tokio runtime for background services and D-Bus
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");

    let _tokio_guard = tokio_runtime.enter();

    // Libadwaita Application
    let app = Application::builder()
        .application_id("org.kepler.Shell")
        .build();

    app.connect_activate(move |app| {
        build_shell_ui(app);
    });

    app.run_with_args(&Vec::<String>::new());
}

fn handle_ipc_client(args: &[String]) {
    let cmd_str = if args[0] == "msg" && args.len() > 1 {
        args[1].as_str()
    } else {
        args[0].as_str()
    };

    let cmd = match cmd_str {
        "toggle-start" | "--toggle-start" => IpcCommand::ToggleStartMenu,
        "toggle-cc" | "--toggle-cc" => IpcCommand::ToggleControlCenter,
        "reload" | "--reload" => IpcCommand::ReloadConfig,
        "quit" | "--quit" => IpcCommand::Quit,
        s if s.starts_with("osd:") => IpcCommand::ShowOsd(s[4..].to_string()),
        s if s.starts_with("snap:") => IpcCommand::SnapActive(s[5..].to_string()),
        _ => {
            eprintln!("Unknown IPC command: {}", cmd_str);
            eprintln!("Valid commands: toggle-start, toggle-cc, reload, quit, osd:<text>, snap:<position>");
            std::process::exit(1);
        }
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(IpcClient::send_command(cmd)) {
        eprintln!("Error sending IPC command: {}", e);
        std::process::exit(1);
    }
}

fn build_shell_ui(app: &Application) {
    let (config, config_path) = load_config();

    // Load CSS and Theme Provider
    let css_provider = gtk4::CssProvider::new();
    ThemeManager::apply(&css_provider, &config.theme.palette);

    // Initialize Compositor Backend
    let backend: Arc<dyn CompositorBackend> = if config.general.compositor_backend == "mutter" {
        Arc::new(MutterDbusBackend::new())
    } else if config.general.compositor_backend == "wayland" {
        Arc::new(WaylandLayerBackend::new())
    } else {
        let mutter = MutterDbusBackend::new();
        if mutter.is_available() {
            Arc::new(mutter)
        } else {
            Arc::new(WaylandLayerBackend::new())
        }
    };

    // Communication Channels (Async to GTK Main Context)
    let (audio_tx, audio_rx) = async_channel::unbounded::<AudioState>();
    let (net_tx, net_rx) = async_channel::unbounded::<NetworkState>();
    let (power_tx, power_rx) = async_channel::unbounded::<PowerState>();
    let (mpris_tx, mpris_rx) = async_channel::unbounded::<Option<MprisState>>();
    let (notif_tx, notif_rx) = async_channel::unbounded::<NotificationItem>();
    let (tray_tx, _tray_rx) = async_channel::unbounded::<TrayEvent>();
    let (ipc_tx, ipc_rx) = async_channel::unbounded::<IpcCommand>();
    let (cfg_tx, cfg_rx) = async_channel::unbounded::<Config>();

    // Start Services on Tokio Runtime
    AudioService::new(audio_tx).start();
    NetworkService::new(net_tx).start();
    PowerService::new(power_tx).start();
    MprisService::new(mpris_tx).start();
    NotificationService::new(notif_tx).start();
    TrayService::new(tray_tx).start();
    IpcServer::start(ipc_tx);
    let _watcher = start_config_watcher(config_path, cfg_tx).ok();

    // Create UI Surfaces
    let osd_win = Arc::new(OsdWindow::new(app.upcast_ref()));
    let start_menu_win = Arc::new(StartMenuWindow::new(app.upcast_ref(), &config));
    let control_center_win = Arc::new(ControlCenterWindow::new(app.upcast_ref(), &config));
    let taskbar_win = Arc::new(TaskbarWindow::new(
        app.upcast_ref(),
        &config,
        backend.clone(),
        start_menu_win.clone(),
        control_center_win.clone(),
    ));

    // Periodic Dock Refresh
    let tb_clone = taskbar_win.clone();
    let bk_clone = backend.clone();
    glib::timeout_add_local(Duration::from_millis(800), move || {
        tb_clone.refresh_dock(bk_clone.as_ref());
        glib::ControlFlow::Continue
    });

    // Spawn async listeners on GLib MainContext (Zero Main Thread Blocking)
    let tb_a = taskbar_win.clone();
    let cc_a = control_center_win.clone();
    let osd_a = osd_win.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = audio_rx.recv().await {
            tb_a.update_audio(&state);
            cc_a.update_audio(&state);
            let frac = (state.volume as f64 / 100.0).clamp(0.0, 1.0);
            let icon = if state.is_muted {
                "audio-volume-muted-symbolic"
            } else {
                "audio-volume-high-symbolic"
            };
            osd_a.show_value(icon, &format!("{}%", state.volume), frac);
        }
    });

    let tb_n = taskbar_win.clone();
    let cc_n = control_center_win.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = net_rx.recv().await {
            tb_n.update_network(&state);
            cc_n.update_network(&state);
        }
    });

    let tb_p = taskbar_win.clone();
    let cc_p = control_center_win.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = power_rx.recv().await {
            tb_p.update_power(&state);
            cc_p.update_power(&state);
        }
    });

    let cc_m = control_center_win.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = mpris_rx.recv().await {
            cc_m.update_mpris(state);
        }
    });

    let osd_notif = osd_win.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(notif) = notif_rx.recv().await {
            osd_notif.show_value(&notif.icon, &notif.summary, 1.0);
        }
    });

    // Config Watcher listener
    let css_p = css_provider.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(new_cfg) = cfg_rx.recv().await {
            ThemeManager::apply(&css_p, &new_cfg.theme.palette);
        }
    });

    // IPC Command listener
    let sm_ipc = start_menu_win.clone();
    let cc_ipc = control_center_win.clone();
    let osd_ipc = osd_win.clone();
    let bk_ipc = backend.clone();
    let css_ipc = css_provider.clone();
    let cfg_ipc = config.clone();
    let app_ipc = app.clone();

    glib::MainContext::default().spawn_local(async move {
        while let Ok(cmd) = ipc_rx.recv().await {
            match cmd {
                IpcCommand::ToggleStartMenu => {
                    cc_ipc.hide();
                    sm_ipc.toggle();
                }
                IpcCommand::ToggleControlCenter => {
                    sm_ipc.hide();
                    cc_ipc.toggle();
                }
                IpcCommand::ReloadConfig => {
                    ThemeManager::apply(&css_ipc, &cfg_ipc.theme.palette);
                }
                IpcCommand::ShowOsd(text) => {
                    osd_ipc.show_value("dialog-information-symbolic", &text, 1.0);
                }
                IpcCommand::SnapActive(pos_str) => {
                    let pos = match pos_str.as_str() {
                        "left" => SnapPosition::Left,
                        "right" => SnapPosition::Right,
                        "top-left" => SnapPosition::TopLeft,
                        "top-right" => SnapPosition::TopRight,
                        "bottom-left" => SnapPosition::BottomLeft,
                        "bottom-right" => SnapPosition::BottomRight,
                        _ => SnapPosition::Maximize,
                    };
                    let wins = bk_ipc.get_windows();
                    if let Some(active) = wins.iter().find(|w| w.is_active) {
                        let _ = bk_ipc.snap_window(&active.id, pos);
                    }
                }
                IpcCommand::Quit => {
                    app_ipc.quit();
                }
            }
        }
    });
}
