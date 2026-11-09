use std::process::Command;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, PartialEq)]
pub struct AudioState {
    pub volume: u32,
    pub is_muted: bool,
    pub output_name: String,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            volume: 50,
            is_muted: false,
            output_name: "Default Output".to_string(),
        }
    }
}

pub struct AudioService {
    sender: async_channel::Sender<AudioState>,
}

impl AudioService {
    pub fn new(sender: async_channel::Sender<AudioState>) -> Self {
        Self { sender }
    }

    pub fn start(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut last_state = AudioState::default();
            loop {
                let state = Self::query_state();
                if state != last_state {
                    last_state = state.clone();
                    let _ = sender.send(state).await;
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
    }

    pub fn query_state() -> AudioState {
        // Query via wpctl (PipeWire)
        if let Ok(output) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                // format: "Volume: 0.65 [MUTED]"
                let is_muted = text.contains("[MUTED]");
                let vol_val = text
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.5);
                let volume = (vol_val * 100.0).round().min(150.0) as u32;

                return AudioState {
                    volume,
                    is_muted,
                    output_name: "PipeWire Output".to_string(),
                };
            }
        }

        // Fallback to pactl (PulseAudio)
        if let Ok(output) = Command::new("pactl").args(["get-sink-volume", "@DEFAULT_SINK@"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(pct) = text.split('%').next().and_then(|s| s.split_whitespace().last()) {
                    let volume = pct.parse::<u32>().unwrap_or(50);
                    return AudioState {
                        volume,
                        is_muted: false,
                        output_name: "PulseAudio Output".to_string(),
                    };
                }
            }
        }

        AudioState::default()
    }

    pub fn set_volume(percent: u32) {
        tokio::spawn(async move {
            let fraction = (percent as f32 / 100.0).to_string();
            let _ = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &fraction])
                .output();
        });
    }

    pub fn toggle_mute() {
        tokio::spawn(async move {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .output();
        });
    }
}
