use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

const INPUT_BY_ID: &str = "/dev/input/by-id/";
const INPUT_EVENT_SIZE: usize = 24;
const POLL_TIMEOUT_MS: i32 = 100; // Wait up to 100ms per device for activity
const RESCAN_INTERVAL: Duration = Duration::from_secs(30); // Pick up reconnected devices

// Linux input event structure
// See: https://www.kernel.org/doc/html/latest/input/input.html
#[repr(C)]
pub struct InputEvent {
    pub time: libc::timeval,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

// Event types from linux/input-event-codes.h
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;

#[derive(Debug)]
pub struct InputDevices {
    devices: Vec<(String, File)>,
    last_scan: Instant,
}

impl InputDevices {
    pub fn new() -> Self {
        InputDevices {
            devices: scan_devices(),
            last_scan: Instant::now(),
        }
    }

    // Re-scan /dev/input/by-id/ and add back any device not currently open
    // (e.g. reconnected after a disconnect that dropped it from the list).
    fn rescan(&mut self) {
        for (path, file) in scan_devices() {
            if !self.devices.iter().any(|(p, _)| p == &path) {
                warn!("Reconnected device: {}", path);
                self.devices.push((path, file));
            }
        }
        self.last_scan = Instant::now();
    }

    // Go over the devices and see if any of them are active
    // Uses poll() to wait for events without burning CPU
    pub fn is_active(&mut self) -> bool {
        if self.last_scan.elapsed() >= RESCAN_INTERVAL {
            self.rescan();
        }

        let mut dead: Vec<String> = vec![];
        let mut active = false;

        for (path, device) in &self.devices {
            let fd = device.as_raw_fd();

            // Wait for activity on this device with timeout
            let mut fds = [libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            }];

            let result = unsafe { libc::poll(fds.as_mut_ptr(), 1, POLL_TIMEOUT_MS) };

            if result == -1 {
                let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);
                if errno == libc::ENODEV {
                    warn!("Device disconnected, dropping: {}", path);
                    dead.push(path.clone());
                } else {
                    warn!("poll error on fd {}: {}", fd, io::Error::last_os_error());
                }
                continue;
            }

            if result == 0 {
                // No activity within timeout - move to next device
                debug!("No activity on fd={}", fd);
                continue;
            }

            // Activity detected - drain ALL available events from this device
            let mut buffer = [0u8; INPUT_EVENT_SIZE];
            loop {
                let n = unsafe {
                    libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len())
                };

                if n == -1 {
                    let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                        // No more events to read
                    } else if errno == libc::ENODEV {
                        warn!("Device disconnected, dropping: {}", path);
                        dead.push(path.clone());
                    } else {
                        warn!("read error on fd {}: {}", fd, io::Error::last_os_error());
                    }
                    break;
                }

                if n == INPUT_EVENT_SIZE as isize {
                    let event = unsafe { &*(buffer.as_ptr() as *const InputEvent) };

                    // Check for actual input events: key presses, mouse movement, etc.
                    // EV_KEY (0x01) = keyboard/mouse buttons
                    // EV_REL (0x02) = relative axis (mouse movement)
                    // We ignore EV_SYN (0x00) synchronization events
                    if (event.type_ == EV_KEY || event.type_ == EV_REL) && event.value != 0 {
                        debug!(
                            "device: {:?} ACTUAL event type={} code={} value={}",
                            device, event.type_, event.code, event.value
                        );
                        active = true;
                        break;
                    }
                }
            }

            if active {
                break;
            }
        }

        if !dead.is_empty() {
            self.devices.retain(|(path, _)| !dead.contains(path));
        }

        active
    }
}

// Scan /dev/input/by-id/ for keyboard/mouse device nodes and open them.
fn scan_devices() -> Vec<(String, File)> {
    let entries = fs::read_dir(INPUT_BY_ID).expect("Could not read devices");
    let mut input: Vec<(String, File)> = vec![];
    for entry in entries {
        let loc = entry.unwrap().path().into_os_string().into_string().unwrap();
        if loc.contains("kbd") || loc.contains("mouse") {
            input.push((loc.clone(), open_device(&loc)));
        }
    }
    input
}

fn open_device(path: &str) -> File {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .expect("Could not open device")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_device_structure() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_device_input");

        if !test_file.exists() {
            std::fs::write(&test_file, "").ok();
        }

        if test_file.exists() {
            let result = std::panic::catch_unwind(|| {
                open_device(test_file.to_str().unwrap());
            });

            let _ = std::fs::remove_file(&test_file);

            assert!(result.is_ok(), "open_device should work with readable file");
        }
    }
}
