mod devices;
mod user;

use crate::devices::InputDevices;
use crate::user::{State, User};
use std::{thread::sleep, time::Duration};

fn main() {
    let devices = InputDevices::new();
    let mut user = User::new();

    user.write_state_to_file();

    // Wait this ammount of time before marking the user idle
    let inactive_cutoff = Duration::new(60, 0);

    loop {
        let device_state = devices.is_active();

        match (&user.state, device_state) {
            // User was idle and became active
            (State::Idle, true) => user.set_state(State::Active),
            // User was active and became idle
            (State::Active, false) => {
                // Wait a bit before marking it as idle, maybe the user is thinking (:
                if user.time_in_current_state() >= inactive_cutoff {
                    user.set_state(State::Idle)
                }
            }
            // For all other cases, no change
            _ => user.write_state_to_file(),
        };

        sleep(Duration::from_millis(1000));
    }
}
