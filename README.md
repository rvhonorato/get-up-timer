# get-up-timer

![Crates.io License](https://img.shields.io/crates/l/get-up-timer)
![Crates.io Version](https://img.shields.io/crates/v/get-up-timer)
![Crates.io Total Downloads](https://img.shields.io/crates/d/get-up-timer)



A simple daemon that monitors your keyboard and mouse activity and reminds you to take breaks.

## Motivation

I often lose track of time and forget to take breaks, so I spent way too much time writing this daemon to keep track of my activity and remind me when its time to take a break! (:

Instead of checking for activity indirectly I decided to just use the `/dev/input/by-id` interface to measure activity directly. This daemon can identify mouse and keyboard devices so it should be compatible anywhere (at least on Linux).

## What it does

The daemon tracks your input device activity and cycles through three states:

- **Active** - you're actively using your computer
- **Idle** - you haven't touched anything for 5 minutes
- **Alert** - you've been active for 1 hour without a break (time to get up!)

It writes the current state to `/tmp/user_state` as JSON, so you can display it in your status bar (something like Waybar).

## Setup

Just run the standard cargo build command:

```bash
cargo build --release
```

You'll find the binary at `target/release/get-up-timer`

Since the daemon needs root access to read from `/dev/input/by-id/`, you'll want to run it as a system service.

Create `/etc/systemd/system/get-up-timer.service`:

```ini
[Unit]
Description=Get Up Timer
After=multi-user.target

[Service]
ExecStart=/path/to/get-up-timer
# For testing with debug mode:
# ExecStart=/path/to/get-up-timer --debug
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Then enable and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable get-up-timer
sudo systemctl start get-up-timer
```

You can check if it's running with:

```bash
sudo systemctl status get-up-timer
```

## Waybar integration

If you're using Waybar, just add this to your config:

```json
 "custom/user-state": {
    "exec": "cat /tmp/user_state",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "tooltip": true,
    "tooltip-format": "{}",
    "markup": true,
  },
```

## Configuration

Right now the timing values are hardcoded in `src/main.rs`:

- Alert after 1 hour of activity
- Break duration: 5 minutes
- Idle timeout: 5 minutes

You can edit these and recompile if you want different timings!

## Output format

The state file is just JSON with Pango markup for pretty colors:

```json
{
  "text": "<span foreground='#a6e3a1'> ● </span>",
  "tooltip": "Active: 00:15:42"
}
```

The different states look like this:

- **Active**: green dot
- **Idle**: yellow dot
- **Alert**: orange "GET UP" text (hard to miss!)
