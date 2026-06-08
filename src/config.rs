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
    pub alert_after: String,
    #[serde(default = "default_break_after")]
    pub break_after: String,
    #[serde(default = "default_idle_after")]
    pub idle_after: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    #[serde(default)]
    pub sound_path: Option<String>,
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
}

// Default values
fn default_alert_after() -> String {
    "1h".to_string()
}
fn default_break_after() -> String {
    "5m".to_string()
}
fn default_idle_after() -> String {
    "30s".to_string()
}
fn default_true() -> bool {
    true
}

fn parse_duration(s: &str) -> Duration {
    let s = s.trim();
    if let Some(suffix) = s.strip_suffix('s') && let Ok(seconds) = suffix.parse::<u64>() {
        return Duration::from_secs(seconds);
    } else if let Some(suffix) = s.strip_suffix('m') && let Ok(minutes) = suffix.parse::<u64>() {
        return Duration::from_secs(minutes * 60);
    } else if let Some(suffix) = s.strip_suffix('h') && let Ok(hours) = suffix.parse::<u64>() {
        return Duration::from_secs(hours * 3600);
    }
    Duration::from_secs(0)
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            alert_after: default_alert_after(),
            break_after: default_break_after(),
            idle_after: default_idle_after(),
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            sound_enabled: default_true(),
            sound_path: None,
            desktop_notifications: default_true(),
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
        parse_duration(&self.timing.alert_after)
    }

    pub fn break_duration(&self) -> Duration {
        parse_duration(&self.timing.break_after)
    }

    pub fn idle_duration(&self) -> Duration {
        parse_duration(&self.timing.idle_after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.timing.alert_after, "1h");
        assert_eq!(config.timing.break_after, "5m");
        assert_eq!(config.timing.idle_after, "30s");
        assert!(config.notifications.sound_enabled);
        assert!(config.notifications.desktop_notifications);
    }

    #[test]
    fn test_alert_duration() {
        let config = Config::default();
        assert_eq!(config.alert_duration(), Duration::from_secs(3600));
    }

    #[test]
    fn test_break_duration() {
        let config = Config::default();
        assert_eq!(config.break_duration(), Duration::from_secs(300));
    }

    #[test]
    fn test_idle_duration() {
        let config = Config::default();
        assert_eq!(config.idle_duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_custom_config() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("get_up_timer_test.toml");

        let content = r#"
[timing]
alert_after = "45m"
break_after = "10m"
idle_after = "3m"

[notifications]
sound_enabled = false
desktop_notifications = true
snooze_minutes = 20
"#;

        fs::write(&config_path, content).unwrap();

        let config = Config::load(Some(&config_path));

        assert_eq!(config.alert_duration(), Duration::from_secs(45 * 60));
        assert_eq!(config.break_duration(), Duration::from_secs(10 * 60));
        assert_eq!(config.idle_duration(), Duration::from_secs(3 * 60));
        assert!(!config.notifications.sound_enabled);
        assert!(config.notifications.desktop_notifications);

        fs::remove_file(&config_path).ok();
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("10s"), Duration::from_secs(10));
        assert_eq!(parse_duration("1m"), Duration::from_secs(60));
        assert_eq!(parse_duration("2h"), Duration::from_secs(7200));
        assert_eq!(parse_duration("30"), Duration::from_secs(0));
    }
}
