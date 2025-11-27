use std::fs::OpenOptions;
use std::fs::{self};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::time::Instant;
use std::{fs::File, thread::sleep, time::Duration};

const INPUT_BY_ID: &str = "/dev/input/by-id/";
const INPUT_EVENT_SIZE: usize = 24;

#[derive(Debug)]
struct InputDevices(Vec<File>);

impl InputDevices {
    fn new() -> Self {
        let devices = fs::read_dir(INPUT_BY_ID).expect("Could not read devices");
        let mut input: Vec<File> = vec![];
        for path in devices {
            let loc = path.unwrap().path().into_os_string().into_string().unwrap();
            // NOTE: Use `mouse` here to also track the mouse
            if loc.contains("kbd") {
                input.push(open_device(&loc));
            }
        }
        InputDevices(input)
    }

    // Go over the devices and see if any of them are active
    fn is_active(&self) -> bool {
        for device in &self.0 {
            let mut buffer = [0u8; INPUT_EVENT_SIZE];
            let fd = device.as_raw_fd();

            let result =
                unsafe { libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len()) };

            match result {
                n if n > 0 => {
                    return true;
                }
                -1 => {
                    // Check errno
                    let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);

                    if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                        continue;
                    } else {
                        eprintln!("Error reading from device: {}", io::Error::last_os_error());
                    }
                }
                _ => {
                    println!("something weird happened")
                }
            }
        }

        return false;
    }
}

fn open_device(path: &str) -> File {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .expect("Could not open device")
}

#[derive(Debug)]
struct User {
    state: State,
    updated: Instant,
}

impl User {
    fn new() -> Self {
        User {
            state: State::Idle,
            updated: Instant::now(),
        }
    }

    fn set_state(&mut self, new_state: State) {
        self.state = new_state;
        self.updated = Instant::now();

        println!("Setting {:?}", self.state);
    }

    fn time_in_current_state(&self) -> Duration {
        Instant::now().duration_since(self.updated)
    }
}

#[derive(PartialEq, Debug)]
enum State {
    Active,
    Idle,
}

fn main() {
    let devices = InputDevices::new();

    let mut user = User::new();

    // Wait this ammount of time before marking the user idle
    let inactive_cutoff = Duration::new(60, 0);

    loop {
        let device_state = devices.is_active();

        match (&user.state, device_state) {
            // User was idle and became active
            (State::Idle, true) => {
                println!("{:?}", user.time_in_current_state());
                user.set_state(State::Active)
            }
            // User was active and became idle
            (State::Active, false) => {
                // Wait a bit before marking it as idle, maybe the user is thinking (:
                if user.time_in_current_state() >= inactive_cutoff {
                    println!("{:?}s passed, the user is idle!", inactive_cutoff);
                    user.set_state(State::Idle)
                } else {
                    println!("User is not touching anything...")
                }
            }
            // For all other cases, no change
            _ => (),
        };

        sleep(Duration::from_millis(100));
    }
}
