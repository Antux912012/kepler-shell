use tokio::time::{sleep, Duration};
use zbus::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkState {
    pub is_connected: bool,
    pub is_wifi_enabled: bool,
    pub ssid: Option<String>,
    pub signal_strength: u8,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            is_connected: false,
            is_wifi_enabled: true,
            ssid: None,
            signal_strength: 0,
        }
    }
}

pub struct NetworkService {
    sender: async_channel::Sender<NetworkState>,
}

impl NetworkService {
    pub fn new(sender: async_channel::Sender<NetworkState>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut last_state = NetworkState::default();
            loop {
                let state = Self::query_state().await;
                if state != last_state {
                    last_state = state.clone();
                    let _ = sender.send(state).await;
                }
                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    async fn query_state() -> NetworkState {
        let Ok(conn) = Connection::system().await else {
            return NetworkState::default();
        };

        // Query NM connectivity
        let connectivity: Result<(zbus::zvariant::OwnedValue,), zbus::Error> = conn
            .call_method(
                Some("org.freedesktop.NetworkManager"),
                "/org/freedesktop/NetworkManager",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.NetworkManager", "Connectivity"),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let is_connected = match connectivity {
            Ok((val,)) => {
                let c = u32::try_from(val).unwrap_or(0);
                c >= 3 // NM_CONNECTIVITY_PORTAL(3) or FULL(4)
            }
            Err(_) => false,
        };

        // Query WirelessEnabled
        let wifi_enabled: Result<(zbus::zvariant::OwnedValue,), zbus::Error> = conn
            .call_method(
                Some("org.freedesktop.NetworkManager"),
                "/org/freedesktop/NetworkManager",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.NetworkManager", "WirelessEnabled"),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let is_wifi_enabled = match wifi_enabled {
            Ok((val,)) => bool::try_from(val).unwrap_or(true),
            Err(_) => true,
        };

        let ssid = if is_connected {
            Some("Connected Network".to_string())
        } else {
            None
        };

        NetworkState {
            is_connected,
            is_wifi_enabled,
            ssid,
            signal_strength: if is_connected { 85 } else { 0 },
        }
    }

    pub fn toggle_wifi() {
        tokio::spawn(async move {
            if let Ok(conn) = Connection::system().await {
                let current: Result<(zbus::zvariant::OwnedValue,), _> = conn
                    .call_method(
                        Some("org.freedesktop.NetworkManager"),
                        "/org/freedesktop/NetworkManager",
                        Some("org.freedesktop.DBus.Properties"),
                        "Get",
                        &("org.freedesktop.NetworkManager", "WirelessEnabled"),
                    )
                    .await
                    .and_then(|r| r.body().deserialize());

                if let Ok((val,)) = current {
                    let enabled = bool::try_from(val).unwrap_or(true);
                    let target = !enabled;
                    let target_val = zbus::zvariant::Value::from(target);
                    let _ = conn
                        .call_method(
                            Some("org.freedesktop.NetworkManager"),
                            "/org/freedesktop/NetworkManager",
                            Some("org.freedesktop.DBus.Properties"),
                            "Set",
                            &("org.freedesktop.NetworkManager", "WirelessEnabled", target_val),
                        )
                        .await;
                }
            }
        });
    }
}
