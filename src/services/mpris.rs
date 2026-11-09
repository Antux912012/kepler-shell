use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use zbus::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct MprisState {
    pub player_bus: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub is_playing: bool,
}

impl Default for MprisState {
    fn default() -> Self {
        Self {
            player_bus: String::new(),
            title: "No Media Playing".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            is_playing: false,
        }
    }
}

pub struct MprisService {
    sender: async_channel::Sender<Option<MprisState>>,
}

impl MprisService {
    pub fn new(sender: async_channel::Sender<Option<MprisState>>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut last_state = None;
            loop {
                let state = Self::query_active_player().await;
                if state != last_state {
                    last_state = state.clone();
                    let _ = sender.send(state).await;
                }
                sleep(Duration::from_millis(1500)).await;
            }
        });
    }

    async fn query_active_player() -> Option<MprisState> {
        let conn = Connection::session().await.ok()?;
        let names: (Vec<String>,) = conn
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .await
            .ok()?
            .body()
            .deserialize()
            .ok()?;

        let mpris_bus = names
            .0
            .into_iter()
            .find(|n| n.starts_with("org.mpris.MediaPlayer2."))?;

        // Query PlaybackStatus
        let status: Result<(zbus::zvariant::OwnedValue,), _> = conn
            .call_method(
                Some(mpris_bus.as_str()),
                "/org/mpris/MediaPlayer2",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let is_playing = match status {
            Ok((val,)) => {
                if let Ok(s) = String::try_from(val) {
                    s == "Playing"
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        // Query Metadata
        let meta_res: Result<(HashMap<String, zbus::zvariant::OwnedValue>,), _> = conn
            .call_method(
                Some(mpris_bus.as_str()),
                "/org/mpris/MediaPlayer2",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.mpris.MediaPlayer2.Player", "Metadata"),
            )
            .await
            .and_then(|r| r.body().deserialize());

        let mut title = "Unknown Title".to_string();
        let mut artist = "".to_string();
        let mut album = "".to_string();

        if let Ok((map,)) = meta_res {
            if let Some(t) = map.get("xesam:title") {
                if let Ok(s) = String::try_from(t.clone()) {
                    title = s;
                }
            }
            if let Some(artists) = map.get("xesam:artist") {
                if let Ok(arr) = Vec::<String>::try_from(artists.clone()) {
                    artist = arr.join(", ");
                } else if let Ok(s) = String::try_from(artists.clone()) {
                    artist = s;
                }
            }
            if let Some(alb) = map.get("xesam:album") {
                if let Ok(s) = String::try_from(alb.clone()) {
                    album = s;
                }
            }
        }

        Some(MprisState {
            player_bus: mpris_bus,
            title,
            artist,
            album,
            is_playing,
        })
    }

    pub fn play_pause(player_bus: String) {
        tokio::spawn(async move {
            if let Ok(conn) = Connection::session().await {
                let _ = conn
                    .call_method(
                        Some(player_bus.as_str()),
                        "/org/mpris/MediaPlayer2",
                        Some("org.mpris.MediaPlayer2.Player"),
                        "PlayPause",
                        &(),
                    )
                    .await;
            }
        });
    }

    pub fn next(player_bus: String) {
        tokio::spawn(async move {
            if let Ok(conn) = Connection::session().await {
                let _ = conn
                    .call_method(
                        Some(player_bus.as_str()),
                        "/org/mpris/MediaPlayer2",
                        Some("org.mpris.MediaPlayer2.Player"),
                        "Next",
                        &(),
                    )
                    .await;
            }
        });
    }

    pub fn previous(player_bus: String) {
        tokio::spawn(async move {
            if let Ok(conn) = Connection::session().await {
                let _ = conn
                    .call_method(
                        Some(player_bus.as_str()),
                        "/org/mpris/MediaPlayer2",
                        Some("org.mpris.MediaPlayer2.Player"),
                        "Previous",
                        &(),
                    )
                    .await;
            }
        });
    }
}
