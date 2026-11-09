use std::sync::{Arc, Mutex};
use zbus::blocking::Connection;

use super::{CompositorBackend, SnapPosition, WindowInfo, WorkspaceInfo};

const BUS_NAME: &str = "org.gnome.Shell.Extensions.KeplerBridge";
const OBJECT_PATH: &str = "/org/gnome/Shell/Extensions/KeplerBridge";
const INTERFACE: &str = "org.gnome.Shell.Extensions.KeplerBridge";

#[derive(Clone)]
pub struct MutterDbusBackend {
    connection: Option<Connection>,
    cached_windows: Arc<Mutex<Vec<WindowInfo>>>,
    active_workspace: Arc<Mutex<u32>>,
    total_workspaces: Arc<Mutex<u32>>,
}

impl MutterDbusBackend {
    pub fn new() -> Self {
        let conn = Connection::session().ok();
        Self {
            connection: conn,
            cached_windows: Arc::new(Mutex::new(Vec::new())),
            active_workspace: Arc::new(Mutex::new(0)),
            total_workspaces: Arc::new(Mutex::new(1)),
        }
    }

    pub fn is_available(&self) -> bool {
        self.connection.is_some()
    }
}

impl CompositorBackend for MutterDbusBackend {
    fn name(&self) -> &'static str {
        "mutter-dbus"
    }

    fn get_windows(&self) -> Vec<WindowInfo> {
        let Some(conn) = &self.connection else {
            return self.cached_windows.lock().unwrap().clone();
        };

        let result: Result<(Vec<(String, String, String, u32, bool, bool)>,), zbus::Error> =
            conn.call_method(
                Some(BUS_NAME),
                OBJECT_PATH,
                Some(INTERFACE),
                "GetWindows",
                &(),
            ).and_then(|reply| reply.body().deserialize());

        match result {
            Ok((tuples,)) => {
                let windows: Vec<WindowInfo> = tuples
                    .into_iter()
                    .map(|(id, title, wm_class, workspace, is_active, is_minimized)| WindowInfo {
                        id,
                        title,
                        wm_class,
                        workspace,
                        is_active,
                        is_minimized,
                    })
                    .collect();
                *self.cached_windows.lock().unwrap() = windows.clone();
                windows
            }
            Err(_) => self.cached_windows.lock().unwrap().clone(),
        }
    }

    fn focus_window(&self, id: &str) -> Result<(), String> {
        let conn = self.connection.as_ref().ok_or_else(|| "No D-Bus connection".to_string())?;
        conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "FocusWindow",
            &(id,),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn minimize_window(&self, id: &str) -> Result<(), String> {
        let conn = self.connection.as_ref().ok_or_else(|| "No D-Bus connection".to_string())?;
        conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "MinimizeWindow",
            &(id,),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn close_window(&self, id: &str) -> Result<(), String> {
        let conn = self.connection.as_ref().ok_or_else(|| "No D-Bus connection".to_string())?;
        conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "CloseWindow",
            &(id,),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn snap_window(&self, id: &str, position: SnapPosition) -> Result<(), String> {
        let conn = self.connection.as_ref().ok_or_else(|| "No D-Bus connection".to_string())?;
        conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "SnapWindow",
            &(id, position.as_str()),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        let Some(conn) = &self.connection else {
            let active = *self.active_workspace.lock().unwrap();
            let total = *self.total_workspaces.lock().unwrap();
            return (0..total).map(|i| WorkspaceInfo { index: i, is_active: i == active }).collect();
        };

        let result: Result<(u32, u32), zbus::Error> = conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "GetWorkspaces",
            &(),
        ).and_then(|reply| reply.body().deserialize());

        if let Ok((active, count)) = result {
            *self.active_workspace.lock().unwrap() = active;
            *self.total_workspaces.lock().unwrap() = count;
            (0..count).map(|i| WorkspaceInfo { index: i, is_active: i == active }).collect()
        } else {
            let active = *self.active_workspace.lock().unwrap();
            let total = *self.total_workspaces.lock().unwrap();
            (0..total).map(|i| WorkspaceInfo { index: i, is_active: i == active }).collect()
        }
    }

    fn switch_workspace(&self, index: u32) -> Result<(), String> {
        let conn = self.connection.as_ref().ok_or_else(|| "No D-Bus connection".to_string())?;
        conn.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "SwitchWorkspace",
            &(index,),
        ).map_err(|e| e.to_string())?;
        *self.active_workspace.lock().unwrap() = index;
        Ok(())
    }
}
