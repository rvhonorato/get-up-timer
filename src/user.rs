use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::time::Duration;
use std::time::Instant;

const ICON_FILE: &str = "/tmp/get-up-timer_icon";
const ELAPSED_FILE: &str = "/tmp/get-up-timer_elapsed";

#[derive(Debug)]
pub struct User {
    pub state: State,
    updated: Instant,
    icon_file: File,
    elapsed_file: File,
}

impl User {
    pub fn new() -> Self {
        let icon_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(ICON_FILE)
            .expect("Failed to open icon file");
        let elapsed_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(ELAPSED_FILE)
            .expect("Failed to open elapsed file");
        User {
            state: State::Idle,
            updated: Instant::now(),
            icon_file,
            elapsed_file,
        }
    }

    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
        self.updated = Instant::now();
    }

    pub fn time_in_current_state(&self) -> Duration {
        Instant::now().duration_since(self.updated)
    }

    pub fn message(&self) -> String {
        match self.state {
            State::Active => "<span foreground='#a6e3a1'> ● </span>".to_string(),
            State::Idle => "<span foreground='#f9e2af'> ○ </span>".to_string(),
            State::Alert => {
                "<span foreground='#fab387' weight='bold' size='x-large'>GET UP</span>".to_string()
            }
        }
    }

    pub fn elapsed_message(&self) -> String {
        let elapsed = self.time_in_current_state();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;
        let timestamp = format!(" {:02}:{:02}:{:02} ", hours, minutes, seconds);

        match self.state {
            State::Active => format!("<span foreground='#a6e3a1'>{}</span>", timestamp),
            State::Idle => format!("<span foreground='#f9e2af'>{}</span>", timestamp),
            State::Alert => format!(
                "<span foreground='#fab387' weight='bold' size='x-large'>{}</span>",
                timestamp
            ),
        }
    }

    pub fn write_state_to_file(&mut self) {
        let elapsed = self.time_in_current_state();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        let timestamp = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

        let tooltip = match self.state {
            State::Active => format!("Active: {}", timestamp),
            State::Idle => format!("Idle: {}", timestamp),
            State::Alert => format!("Alert: {}", timestamp),
        };

        // Write icon file
        let icon_content = serde_json::json!({
            "text": self.message(),
            "tooltip": tooltip.clone()
        })
        .to_string();

        write_file_content(&mut self.icon_file, &icon_content);

        // Write elapsed file
        let elapsed_content = serde_json::json!({
            "text": self.elapsed_message(),
            "tooltip": tooltip
        })
        .to_string();

        write_file_content(&mut self.elapsed_file, &elapsed_content);
    }
}

fn write_file_content(file: &mut File, content: &str) {
    if let Err(e) = file.seek(SeekFrom::Start(0)) {
        eprintln!("Failed to seek file: {}", e);
        return;
    }

    if let Err(e) = file.write_all(content.as_bytes()) {
        eprintln!("Failed to write file: {}", e);
        return;
    }

    if let Err(e) = file.set_len(content.len() as u64) {
        eprintln!("Failed to truncate file: {}", e);
        return;
    }

    if let Err(e) = file.flush() {
        eprintln!("Failed to flush file: {}", e);
    }
}

#[derive(PartialEq, Debug)]
pub enum State {
    Active,
    Idle,
    Alert,
}
