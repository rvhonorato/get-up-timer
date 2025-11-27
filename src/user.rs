use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::time::Duration;
use std::time::Instant;

const STATE_FILE: &str = "/tmp/user_state";

#[derive(Debug)]
pub struct User {
    pub state: State,
    updated: Instant,
    state_file: File,
}

impl User {
    pub fn new() -> Self {
        let state_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(STATE_FILE)
            .expect("Failed to open state file");
        User {
            state: State::Idle,
            updated: Instant::now(),
            state_file,
        }
    }

    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
        self.updated = Instant::now();
    }

    pub fn time_in_current_state(&self) -> Duration {
        Instant::now().duration_since(self.updated)
    }

    pub fn write_state_to_file(&mut self) {
        let elapsed = self.time_in_current_state();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        let content = format!(
            "{:?} {:02}:{:02}:{:02}\n",
            self.state, hours, minutes, seconds
        );

        // Seek to beginning and overwrite
        if let Err(e) = self.state_file.seek(SeekFrom::Start(0)) {
            eprintln!("Failed to seek state file: {}", e);
            return;
        }

        if let Err(e) = self.state_file.write_all(content.as_bytes()) {
            eprintln!("Failed to write state file: {}", e);
            return;
        }

        // Truncate in case new content is shorter than old content
        if let Err(e) = self.state_file.set_len(content.len() as u64) {
            eprintln!("Failed to truncate state file: {}", e);
            return;
        }

        // Flush to ensure it's written immediately
        if let Err(e) = self.state_file.flush() {
            eprintln!("Failed to flush state file: {}", e);
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum State {
    Active,
    Idle,
}
