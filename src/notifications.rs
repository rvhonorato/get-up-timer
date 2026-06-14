use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::config::Config;
use crate::user::State;

const SNOOZE_FILE: &str = "/tmp/get_up_snooze";
const NOTIFY_SCRIPT_FILE: &str = "/tmp/get-up-timer_notify";
const PAUSE_FILE: &str = "/tmp/get-up-timer_pause_notification";

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
        if self.is_paused() {
            return false;
        }

        if !self.config.notifications.sound_enabled && !self.should_desktop_notify() {
            return false;
        }
        true
    }

    fn is_paused(&self) -> bool {
        Path::new(PAUSE_FILE).exists()
    }

    fn should_desktop_notify(&self) -> bool {
        let path = Path::new(SNOOZE_FILE);
        if path.exists() {
            return false;
        }
        true
    }

    pub fn send_notification(&mut self, state: State) {
        match state {
            State::Active | State::Idle | State::Break => {
                // Active/Idle/Break don't trigger a notification: clear the pending
                // notification script and lift any pause so the next Alert notifies.
                self.clear_notify_script();
                self.clear_pause_file();
            }
            State::Alert => {
                if !self.should_notify() {
                    return;
                }
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

    fn clear_pause_file(&self) {
        let _ = fs::remove_file(PAUSE_FILE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use std::sync::Mutex;

    // Tests that read or write PAUSE_FILE share this lock to avoid racing
    // each other over the same path in /tmp when run in parallel.
    static PAUSE_FILE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_notification_manager_new() {
        let _guard = PAUSE_FILE_LOCK.lock().unwrap();
        let _ = fs::remove_file(PAUSE_FILE);

        let config = Config::default();
        let manager = NotificationManager::new(&config);
        assert!(manager.should_notify());
    }

    #[test]
    fn test_should_notify_logic() {
        let _guard = PAUSE_FILE_LOCK.lock().unwrap();
        let _ = fs::remove_file(PAUSE_FILE);

        let mut config = Config::default();
        config.notifications.sound_enabled = true;
        config.notifications.desktop_notifications = true;

        let manager = NotificationManager::new(&config);
        assert!(manager.should_notify());
    }

    #[test]
    fn test_clear_notify_script() {
        let test_notify = "/tmp/get-up-timer_test_notify_script";
        const ORIGINAL_NOTIFY: &str = "/tmp/get-up-timer_notify";

        fs::write(test_notify, "test").ok();

        if Path::new(ORIGINAL_NOTIFY).exists() {
            fs::remove_file(ORIGINAL_NOTIFY).ok();
        }

        fs::write(ORIGINAL_NOTIFY, "test").ok();

        let manager = NotificationManager::new(&Config::default());
        manager.clear_notify_script();

        assert!(!Path::new(ORIGINAL_NOTIFY).exists());

        let _ = fs::remove_file(test_notify);
    }

    #[test]
    fn test_should_notify_paused() {
        let _guard = PAUSE_FILE_LOCK.lock().unwrap();
        let _ = fs::remove_file(PAUSE_FILE);

        let mut config = Config::default();
        config.notifications.sound_enabled = true;
        config.notifications.desktop_notifications = true;

        let manager = NotificationManager::new(&config);
        assert!(manager.should_notify());

        fs::write(PAUSE_FILE, "").ok();
        assert!(!manager.should_notify());

        let _ = fs::remove_file(PAUSE_FILE);
    }

    #[test]
    fn test_pause_cleared_on_active_idle_and_break() {
        let _guard = PAUSE_FILE_LOCK.lock().unwrap();

        let config = Config::default();
        let mut manager = NotificationManager::new(&config);

        for state in [State::Active, State::Idle, State::Break] {
            fs::write(PAUSE_FILE, "").ok();
            manager.send_notification(state);
            assert!(!Path::new(PAUSE_FILE).exists());
        }
    }

    #[test]
    fn test_notification_state_changes() {
        let _guard = PAUSE_FILE_LOCK.lock().unwrap();
        let _ = fs::remove_file(PAUSE_FILE);

        let mut config = Config::default();
        config.notifications.sound_enabled = false;
        config.notifications.desktop_notifications = false;

        let mut manager = NotificationManager::new(&config);

        manager.send_notification(State::Active);
        assert!(manager.last_notification.is_none());

        manager.send_notification(State::Idle);
        assert!(manager.last_notification.is_none());

        manager.send_notification(State::Alert);
        assert!(manager.last_notification.is_some());

        // Break no longer triggers a notification - it should not change
        // last_notification
        manager.last_notification = None;
        manager.send_notification(State::Break);
        assert!(manager.last_notification.is_none());
    }
}
