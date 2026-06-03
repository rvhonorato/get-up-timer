# get-up-timer

![Crates.io License](https://img.shields.io/crates/l/get-up-timer)
![Crates.io Version](https://img.shields.io/crates/v/get-up-timer)
![Crates.io Total Downloads](https://img.shields.io/crates/d/get-up-timer)

A simple daemon that monitors your keyboard and mouse activity and reminds you to take breaks.

## Motivation

I often lose track of time and forget to take breaks, so I spent way too much time writing this daemon to keep track of my activity and remind me when it's time to take a break! (:

Instead of checking for activity indirectly, I decided to just use the `/dev/input/by-id` interface to measure activity directly. This daemon can identify mouse and keyboard devices so it should be compatible anywhere (at least on Linux).

## What it does

The daemon tracks your input device activity and cycles through three states:

- **Active** - you're actively using your computer
- **Idle** - you haven't touched anything for 5 minutes (configurable)
- **Alert** - you've been active for 1 hour without a break (time to get up!)

## Installation

```bash
cargo install get-up-timer
```

The binary will be installed to `~/.cargo/bin/get-up-timer`.

## Setup

I developed this specifically to be executed as a background service and integrated into `waybar`

### Service

Create `/etc/systemd/system/get-up-timer.service` (obviously update the paths):

```
[Unit]
Description=User Activity Monitor
After=multi-user.target

[Service]
Type=simple
ExecStart=/home/rodrigo/.cargo/bin/get-up-timer /home/rodrigo/.config/get-up-timer/config.toml
Restart=always
RestartSec=5
SupplementaryGroups=input

[Install]
WantedBy=multi-user.target
```

Enable the service

```bash
sudo systemctl daemon-reload
sudo systemctl enable get-up-timer.service --now
```

Note: It must be executed as root to have access to your input devices.

### Waybar

`get-up-timer` will write files to your `/tmp` directory, and these files can be used in `waybar`.

- `get-up-timer_elapsed`: a timestamp display showing for how long user has been in a given state
- `get-up-timer_icon`: a simplified display showing only a green dot or a `GET UP` text
- `get-up-timer_notify`: a notification script

```jsonc
  "custom/get-up-timer": {
    "exec": "cat /tmp/get-up-timer_elapsed",
    "interval": 1,
    "return-type": "json",
    "format": "{text}",
    "tooltip-format": "{tooltip}"
  },
  "custom/get-up-timer-notify": {
    "exec": "[ -e \"/tmp/get-up-timer_notify\" ] && bash /tmp/get-up-timer_notify || echo ''",
    "interval": 60, // In seconds
    "return-type": "string"
  },
```

### Configuration

An example configuration is provided:

```toml
#=======================================================================================
[timing]
# After this many minutes of continuous activity show "GET UP" alert
alert_after_minutes = 60
# While in Alert state: 
#  - after this many minutes of inactivity (idle), clear the alert and return to Idle
break_after_minutes = 5
# After this many minutes of inactivity (no input), transition from Active to Idle state
idle_after_minutes = 5
#=======================================================================================
[notifications]
# Whether sound notifications are enabled
sound_enabled = true
# Path to sound file to play when alert triggers (using paplay)
sound_path = "/usr/share/sounds/alert.wav"
# Whether desktop notifications are enabled (using notify-send)
desktop_notifications = true
#=======================================================================================
```

## Contributing

PRs welcome! See `TODO.md` for ideas.
