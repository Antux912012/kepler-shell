use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use zbus::interface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayItem {
    pub service: String,
    pub title: String,
    pub icon_name: String,
    pub menu_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Added(TrayItem),
    Removed(String),
}

pub struct StatusNotifierWatcher {
    items: Arc<Mutex<HashSet<String>>>,
    sender: async_channel::Sender<TrayEvent>,
}

impl StatusNotifierWatcher {
    pub fn new(sender: async_channel::Sender<TrayEvent>) -> Self {
        Self {
            items: Arc::new(Mutex::new(HashSet::new())),
            sender,
        }
    }
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(&self, service: String) {
        let is_new = {
            let mut items = self.items.lock().unwrap();
            items.insert(service.clone())
        };
        if is_new {
            let item = TrayItem {
                service: service.clone(),
                title: service.clone(),
                icon_name: "application-x-executable".to_string(),
                menu_path: None,
            };
            let _ = self.sender.send(TrayEvent::Added(item)).await;
        }
    }

    async fn register_status_notifier_host(&self, _service: String) {}

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().unwrap().iter().cloned().collect()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }
}

pub struct TrayService {
    sender: async_channel::Sender<TrayEvent>,
}

impl TrayService {
    pub fn new(sender: async_channel::Sender<TrayEvent>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let watcher = StatusNotifierWatcher::new(sender);
            let conn = zbus::connection::Builder::session()
                .and_then(|b| b.name("org.kde.StatusNotifierWatcher"))
                .and_then(|b| b.serve_at("/StatusNotifierWatcher", watcher));

            if let Ok(builder) = conn {
                if let Ok(connection) = builder.build().await {
                    let () = std::future::pending().await;
                    drop(connection);
                }
            }
        });
    }
}
