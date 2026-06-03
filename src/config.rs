use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    pub timing: TimingConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimingConfig {
    #[serde(default = "default_alert_after")]
    pub alert_after_minutes: u64,
    #[serde(default = "default_break_after")]
    pub break_after_minutes: u64,
    #[serde(default = "default_idle_after")]
    pub idle_after_minutes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    #[serde(default)]
    pub sound_path: Option<String>,
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
    /// Snooze duration in minutes when the snooze file is detected.
    #[serde(default = "default_snooze")]
    pub snooze_minutes: u64,
}

// Default values
fn default_alert_after() -> u64 {
    60
}
fn default_break_after() -> u64 {
    5
}
fn default_idle_after() -> u64 {
    5
}
fn default_snooze() -> u64 {
    15
}
fn default_true() -> bool {
    true
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            alert_after_minutes: default_alert_after(),
            break_after_minutes: default_break_after(),
            idle_after_minutes: default_idle_after(),
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            sound_enabled: default_true(),
            sound_path: None,
            desktop_notifications: default_true(),
            snooze_minutes: default_snooze(),
        }
    }
}

fn xdg_config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join("get-up-timer")
}

fn try_load(path: &Path) -> Option<Config> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read config file '{}': {}",
                path.display(),
                e
            );
            return None;
        }
    };
    match toml::from_str(&content) {
        Ok(cfg) => {
            eprintln!("Config loaded from {}", path.display());
            Some(cfg)
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to parse config file '{}': {}",
                path.display(),
                e
            );
            eprintln!("Using defaults.");
            None
        }
    }
}

impl Config {
    /// Load config, searching standard paths when none is explicitly given.
    ///
    /// Search order (first found wins):
    ///   1. Explicit path (if provided)
    ///   2. $XDG_CONFIG_HOME/get-up-timer/config.toml
    ///   3. /etc/get-up-timer/config.toml
    pub fn load(config_path: Option<&Path>) -> Self {
        if let Some(path) = config_path {
            return try_load(path).unwrap_or_default();
        }

        let candidates = [
            xdg_config_path().join("config.toml"),
            PathBuf::from("/etc/get-up-timer/config.toml"),
        ];

        for path in &candidates {
            if let Some(cfg) = try_load(path) {
                return cfg;
            }
        }

        Config::default()
    }

    pub fn alert_duration(&self) -> Duration {
        Duration::from_secs(self.timing.alert_after_minutes * 60)
    }

    pub fn break_duration(&self) -> Duration {
        Duration::from_secs(self.timing.break_after_minutes * 60)
    }

    pub fn idle_duration(&self) -> Duration {
        Duration::from_secs(self.timing.idle_after_minutes * 60)
    }

    pub fn snooze_duration(&self) -> Duration {
        Duration::from_secs(self.notifications.snooze_minutes * 60)
    }
}
