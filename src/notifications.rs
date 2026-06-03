use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::config::Config;
use crate::user::State;

const SNOOZE_FILE: &str = "/tmp/get_up_snooze";
const NOTIFY_SCRIPT_FILE: &str = "/tmp/get-up-timer_notify";

pub struct NotificationManager {
    last_notification: Option<Instant>,
    config: Config,
}

impl NotificationManager {
    pub fn new(config: &Config) -> Self {
        NotificationManager {
            last_notification: None,
            config: config.clone(),
        }
    }

    pub fn should_notify(&self) -> bool {
        if !self.config.notifications.sound_enabled && !self.should_desktop_notify() {
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
        if !self.should_notify() {
            return;
        }

        match state {
            State::Active => {
                // Clear the notification script when returning to Active
                self.clear_notify_script();
            }
            State::Idle => {
                // Clear the notification script when going idle
                self.clear_notify_script();
            }
            State::Alert => {
                self.write_notify_script();
                self.last_notification = Some(Instant::now());
            }
        }
    }

    fn write_notify_script(&self) {
        let mut script = String::new();

        // Add desktop notification command
        if self.should_desktop_notify() && self.config.notifications.desktop_notifications {
            script.push_str(
                "notify-send --app-name=get-up-timer --urgency=critical --category=im.received \"get-up-timer\" \"It is time to take a break! Get up and stretch.\"\n",
            );
        }

        // Add sound command
        if self.config.notifications.sound_enabled {
            let sound_path = self
                .config
                .notifications
                .sound_path
                .as_ref()
                .cloned()
                .unwrap_or_else(|| {
                    "/usr/share/sounds/freedesktop/stereo/phone-outgoing-busy.oga".to_string()
                });
            script.push_str(&format!("paplay {}\n", sound_path));
        }

        // Write the script to file
        if !script.is_empty() && fs::write(NOTIFY_SCRIPT_FILE, script).is_err() {
            eprintln!("Failed to write notification script");
        }
    }

    fn clear_notify_script(&self) {
        let _ = fs::remove_file(NOTIFY_SCRIPT_FILE);
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
