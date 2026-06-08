mod config;
mod devices;
mod notifications;
mod user;

use crate::config::Config;
use crate::devices::InputDevices;
use crate::notifications::NotificationManager;
use crate::user::{State, User};
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting get-up-timer");

    let config_path = std::env::args().nth(1).map(PathBuf::from);
    let config = Config::load(config_path.as_deref());
    info!(
        "Configuration: idle_duration={:?}, alert_duration={:?}, break_duration={:?}",
        config.idle_duration(),
        config.alert_duration(),
        config.break_duration()
    );

    let devices = InputDevices::new();
    let mut user = User::new();
    let mut notifier = NotificationManager::new(&config);

    info!("Initial state: {:?}", user.state);
    user.write_state_to_file();

    let mut last_inactive_time: Option<std::time::Instant> = None;

    loop {
        let device_active = devices.is_active();
        let elapsed_in_state = user.time_in_current_state();

        match (&user.state, device_active) {
            (State::Idle, true) => {
                warn!("State transition: Idle -> Active (device became active)");
                user.set_state(State::Active);
            }

            (State::Idle, false) => {}

            (State::Active, false) => {
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());
                let inactive_elapsed = inactive_start.elapsed();

                info!(
                    "Inactive for {:?} (idle threshold: {:?})",
                    inactive_elapsed,
                    config.idle_duration()
                );

                if inactive_elapsed >= config.idle_duration() {
                    warn!("State transition: Active -> Idle (idle timeout reached)");
                    user.set_state(State::Idle);
                    last_inactive_time = None;
                }
            }

            (State::Active, true) => {
                last_inactive_time = None;

                if elapsed_in_state >= config.alert_duration() {
                    warn!("State transition: Active -> Alert (alert timeout reached)");
                    user.set_state(State::Alert);
                    notifier.send_notification(State::Alert);
                }
            }

            (State::Alert, false) => {
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());
                let inactive_elapsed = inactive_start.elapsed();

                info!(
                    "Alert: Inactive for {:?} (idle threshold: {:?})",
                    inactive_elapsed,
                    config.idle_duration()
                );

                if inactive_elapsed >= config.idle_duration() {
                    warn!("State transition: Alert -> Break (idle timeout reached)");
                    user.set_state(State::Break);
                    notifier.send_notification(State::Break);
                    last_inactive_time = Some(std::time::Instant::now());
                }
            }
            (State::Alert, true) => {
                // User is still active during Alert - extend timer
                last_inactive_time = None;
            }

            (State::Break, true) => {
                // User resumed activity during break - break was interrupted, go back to Alert
                warn!("State transition: Break -> Alert (break interrupted, resumed activity)");
                user.set_state(State::Alert);
                notifier.send_notification(State::Alert);
                last_inactive_time = None;
            }

            (State::Break, false) => {
                // Still on break - check if break duration has elapsed
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());
                let inactive_elapsed = inactive_start.elapsed();

                info!(
                    "On break, inactive for {:?} (break threshold: {:?})",
                    inactive_elapsed,
                    config.break_duration()
                );

                if inactive_elapsed >= config.break_duration() {
                    warn!("State transition: Break -> Idle (break completed)");
                    user.set_state(State::Idle);
                    notifier.send_notification(State::Idle);
                    last_inactive_time = None;
                }
            }
        };

        user.write_state_to_file();
    }
}
