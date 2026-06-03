use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::user::State;

const SNOOZE_FILE: &str = "/tmp/get_up_snooze";

pub struct NotificationManager {
    last_notification: Option<Instant>,
    repeat_interval: Option<Duration>,
    config: Config,
}

impl NotificationManager {
    pub fn new(config: &Config) -> Self {
        let repeat_interval = config
            .notifications
            .repeat_interval_minutes
            .map(|m| Duration::from_secs(m * 60));
        NotificationManager {
            last_notification: None,
            repeat_interval,
            config: config.clone(),
        }
    }

    pub fn should_notify(&self, current_state: &State) -> bool {
        if !self.config.notifications.sound_enabled && !self.should_desktop_notify() {
            return false;
        }

        if *current_state == State::Alert
            && let Some(repeat) = self.repeat_interval
            && let Some(last) = self.last_notification
            && last.elapsed() < repeat
        {
            return false;
        }

        true
    }

    fn should_desktop_notify(&self) -> bool {
        let path = Path::new(SNOOZE_FILE);
        if path.exists() {
            return false;
        }
        true
    }

    pub fn send_notification(&mut self, state: State) {
        if !self.should_notify(&state) {
            return;
        }

        match state {
            State::Active => (),
            State::Idle => (),
            State::Alert => {
                self.send_alert_notification();
                self.last_notification = Some(Instant::now());
            }
        }
    }

    fn send_alert_notification(&self) {
        let (title, message) = (
            "get-up-timer",
            "It's time to take a break! Get up and stretch.",
        );

        if self.should_desktop_notify() {
            self.send_desktop_notification(title, message);
        }

        self.send_sound_notification();
    }

    fn send_desktop_notification(&self, title: &str, message: &str) {
        let result = Command::new("notify-send")
            .arg("--app-name=get-up-timer")
            .arg("--urgency")
            .arg("critical")
            .arg("--category")
            .arg("im.received")
            .arg(title)
            .arg(message)
            .status();

        if let Err(e) = result {
            eprintln!("Failed to send desktop notification: {}", e);
        }
    }

    fn send_sound_notification(&self) {
        if !self.config.notifications.sound_enabled {
            return;
        }

        let sound_path = self
            .config
            .notifications
            .sound_command
            .as_ref()
            .cloned()
            .unwrap_or_else(|| {
                "/usr/share/sounds/freedesktop/stereo/phone-outgoing-busy.oga".to_string()
            });

        let result = Command::new("paplay").arg(sound_path).status();

        match result {
            Ok(status) => {
                if !status.success() {
                    eprintln!("Failed to play sound");
                }
            }
            Err(e) => {
                eprintln!("Failed to play sound: {}", e);
            }
        }
    }
}

pub fn snooze(minutes: u64) {
    let path = Path::new(SNOOZE_FILE);

    match std::fs::write(path, minutes.to_string()) {
        Ok(_) => println!("Snoozed for {} minutes", minutes),
        Err(e) => eprintln!("Failed to snooze: {}", e),
    }
}

pub fn clear_snooze() {
    let path = Path::new(SNOOZE_FILE);
    let _ = std::fs::remove_file(path);
}

pub fn is_snoozed() -> bool {
    let path = Path::new(SNOOZE_FILE);
    !path.exists()
}
