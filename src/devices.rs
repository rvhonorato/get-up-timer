use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

const INPUT_BY_ID: &str = "/dev/input/by-id/";
const INPUT_EVENT_SIZE: usize = 24;

#[derive(Debug)]
pub struct InputDevices(Vec<File>);

impl InputDevices {
    pub fn new() -> Self {
        let devices = fs::read_dir(INPUT_BY_ID).expect("Could not read devices");
        let mut input: Vec<File> = vec![];
        for path in devices {
            let loc = path.unwrap().path().into_os_string().into_string().unwrap();
            // NOTE: Use `mouse` here to also track the mouse
            if loc.contains("kbd") || loc.contains("mouse") {
                input.push(open_device(&loc));
            }
        }
        InputDevices(input)
    }

    // Go over the devices and see if any of them are active
    pub fn is_active(&self) -> bool {
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
