use tokio::time::{sleep, Duration};
use zbus::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct PowerState {
    pub percentage: u8,
    pub is_charging: bool,
    pub has_battery: bool,
    pub power_profile: String,
}

impl Default for PowerState {
    fn default() -> Self {
        Self {
            percentage: 100,
            is_charging: false,
            has_battery: true,
            power_profile: "balanced".to_string(),
        }
    }
}

pub struct PowerService {
    sender: async_channel::Sender<PowerState>,
}

impl PowerService {
    pub fn new(sender: async_channel::Sender<PowerState>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut last_state = PowerState::default();
            loop {
                let state = Self::query_state().await;
                if state != last_state {
                    last_state = state.clone();
                    let _ = sender.send(state).await;
                }
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn query_state() -> PowerState {
        let Ok(conn) = Connection::system().await else {
            return PowerState::default();
        };

        // Query DisplayDevice from UPower
        let display_device: Result<(zbus::zvariant::OwnedObjectPath,), _> = conn
            .call_method(
                Some("org.freedesktop.UPower"),
                "/org/freedesktop/UPower",
                Some("org.freedesktop.UPower"),
                "GetDisplayDevice",
                &(),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let mut percentage = 100u8;
        let mut is_charging = false;
        let mut has_battery = false;

        if let Ok((path,)) = display_device {
            let pct: Result<(zbus::zvariant::OwnedValue,), _> = conn
                .call_method(
                    Some("org.freedesktop.UPower"),
                    path.as_str(),
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.freedesktop.UPower.Device", "Percentage"),
                )
                .await
                .and_then(|r| r.body().deserialize());

            if let Ok((val,)) = pct {
                if let Ok(f) = f64::try_from(val) {
                    percentage = f.round() as u8;
                    has_battery = true;
                }
            }

            let state_res: Result<(zbus::zvariant::OwnedValue,), _> = conn
                .call_method(
                    Some("org.freedesktop.UPower"),
                    path.as_str(),
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.freedesktop.UPower.Device", "State"),
                )
                .await
                .and_then(|r| r.body().deserialize());

            if let Ok((val,)) = state_res {
                if let Ok(s) = u32::try_from(val) {
                    // State: 1 = Charging, 4 = Fully charged
                    is_charging = s == 1 || s == 4;
                }
            }
        }

        // Query power-profiles-daemon
        let profile_res: Result<(zbus::zvariant::OwnedValue,), _> = conn
            .call_method(
                Some("net.hadess.PowerProfiles"),
                "/net/hadess/PowerProfiles",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("net.hadess.PowerProfiles", "ActiveProfile"),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let power_profile = match profile_res {
            Ok((val,)) => String::try_from(val).unwrap_or_else(|_| "balanced".to_string()),
            Err(_) => "balanced".to_string(),
        };

        PowerState {
            percentage,
            is_charging,
            has_battery,
            power_profile,
        }
    }

    pub fn set_power_profile(profile: String) {
        tokio::spawn(async move {
            if let Ok(conn) = Connection::system().await {
                let target_val = zbus::zvariant::Value::from(profile);
                let _ = conn
                    .call_method(
                        Some("net.hadess.PowerProfiles"),
                        "/net/hadess/PowerProfiles",
                        Some("org.freedesktop.DBus.Properties"),
                        "Set",
                        &("net.hadess.PowerProfiles", "ActiveProfile", target_val),
                    )
                    .await;
            }
        });
    }
}
