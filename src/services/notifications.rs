use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use zbus::interface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationItem {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub actions: Vec<String>,
}

pub struct NotificationServer {
    next_id: Arc<AtomicU32>,
    sender: async_channel::Sender<NotificationItem>,
}

impl NotificationServer {
    pub fn new(sender: async_channel::Sender<NotificationItem>) -> Self {
        Self {
            next_id: Arc::new(AtomicU32::new(1)),
            sender,
        }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        _hints: HashMap<String, zbus::zvariant::Value<'_>>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            self.next_id.fetch_add(1, Ordering::SeqCst)
        };

        let item = NotificationItem {
            id,
            app_name,
            summary,
            body,
            icon: app_icon,
            actions,
        };

        let _ = self.sender.send(item).await;
        id
    }

    async fn close_notification(&self, _id: u32) {}

    async fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "actions".to_string(),
            "persistence".to_string(),
        ]
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Kepler Shell Notification Server".to_string(),
            "Kepler".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

pub struct NotificationService {
    sender: async_channel::Sender<NotificationItem>,
}

impl NotificationService {
    pub fn new(sender: async_channel::Sender<NotificationItem>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let server = NotificationServer::new(sender);
            let conn = zbus::connection::Builder::session()
                .and_then(|b| b.name("org.freedesktop.Notifications"))
                .and_then(|b| b.serve_at("/org/freedesktop/Notifications", server));

            if let Ok(builder) = conn {
                if let Ok(connection) = builder.build().await {
                    let () = std::future::pending().await;
                    drop(connection);
                }
            }
        });
    }
}
