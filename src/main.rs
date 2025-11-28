mod devices;
mod user;

use crate::devices::InputDevices;
use crate::user::{State, User};
use std::{thread::sleep, time::Duration};

// ================================================================================
// TODO: Make these configurable
//
// Duration needed to trigger the GET UP alert
const ALERT_TRIGGER: Duration = Duration::from_secs(60 * 60); // 1 hour

// Duration needed to clear the alert, this is your break!
const BREAK_DURATION: Duration = Duration::from_secs(60 * 5); // 5 minutes

// Duration to define that the user is idle
// Note: once the user is set to idle, the active timer will reset, so take this
//  duration here into consideration, not too short but also not too long
const IDLE_TRIGGER: Duration = Duration::from_secs(60 * 5); // 5 minute
// ================================================================================

fn main() {
    let devices = InputDevices::new();
    let mut user = User::new();

    user.write_state_to_file();

    let mut last_inactive_time: Option<std::time::Instant> = None;

    loop {
        let device_state = devices.is_active();

        // println!(
        //     "device state: {} user state: {:?}",
        //     device_state, user.state
        // );

        match (&user.state, device_state) {
            // User was idle and devices became active
            (State::Idle, true) => user.set_state(State::Active),

            // User was active and devices became idle
            (State::Active, false) => {
                // Start tracking inactivity
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());

                // Check if devices have been inactive long enough
                if inactive_start.elapsed() >= IDLE_TRIGGER {
                    user.set_state(State::Idle);
                    last_inactive_time = None;
                }
            }

            // User is active and devices are active
            (State::Active, true) => {
                // Reset the inactivity timer since devices are active
                last_inactive_time = None;

                // Check if its time to trigger the alarm
                if user.time_in_current_state() >= ALERT_TRIGGER {
                    user.set_state(State::Alert);
                }
            }

            // There is an alert and the device are not active
            (State::Alert, false) => {
                let inactive_start = last_inactive_time.get_or_insert(std::time::Instant::now());

                // Check if its time to clear the alarm
                if inactive_start.elapsed() >= BREAK_DURATION {
                    user.set_state(State::Idle);
                }
            }
            // There is an alert and the devices are active
            (State::Alert, true) => {
                // Reset the inactivity timer since devices are active
                last_inactive_time = None;
            }
            // For all other cases, no change
            _ => {
                last_inactive_time = None;
            }
        };

        user.write_state_to_file();

        sleep(Duration::from_millis(10));
    }
}
