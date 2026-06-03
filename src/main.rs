mod config;
mod devices;
mod notifications;
mod user;

use crate::config::Config;
use crate::devices::InputDevices;
use crate::notifications::NotificationManager;
use crate::user::{State, User};
use std::path::PathBuf;
use std::{thread::sleep, time::Duration};

fn main() {
    let config_path = std::env::args().nth(1).map(PathBuf::from);
    let config = Config::load(config_path.as_deref());
    let devices = InputDevices::new();
    let mut user = User::new();
    let mut notifier = NotificationManager::new(&config);

    user.write_state_to_file();

    let mut last_inactive_time: Option<std::time::Instant> = None;

    loop {
        let device_state = devices.is_active();

        match (&user.state, device_state) {
            (State::Idle, true) => user.set_state(State::Active),

            (State::Active, false) => {
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());

                if inactive_start.elapsed() >= config.idle_duration() {
                    user.set_state(State::Idle);
                    last_inactive_time = None;
                }
            }

            (State::Active, true) => {
                last_inactive_time = None;

                if user.time_in_current_state() >= config.alert_duration() {
                    user.set_state(State::Alert);
                    notifier.send_notification(State::Alert);
                }
            }

            (State::Alert, false) => {
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());

                if inactive_start.elapsed() >= config.break_duration() {
                    user.set_state(State::Idle);
                    last_inactive_time = None;
                }
            }
            (State::Alert, true) => {
                last_inactive_time = None;
            }
            _ => {
                last_inactive_time = None;
            }
        };

        user.write_state_to_file();

        sleep(Duration::from_millis(500));
    }
}
