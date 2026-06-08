# get-up-timer

![Crates.io License](https://img.shields.io/crates/l/get-up-timer)
![Crates.io Version](https://img.shields.io/crates/v/get-up-timer)
![Crates.io Total Downloads](https://img.shields.io/crates/d/get-up-timer)

A simple daemon that monitors your keyboard and mouse activity and reminds you to take breaks.

## Motivation

I often lose track of time and forget to take breaks, so I spent way too much time writing this daemon to keep track of my activity and remind me when it's time to take a break! (:

Instead of checking for activity indirectly, I decided to just use the `/dev/input/by-id` interface to measure activity directly. This daemon can identify mouse and keyboard devices so it should be compatible anywhere (at least on Linux).

## What it does

The daemon tracks your input device activity and cycles through four states:

- **Active** - you're actively using your computer
- **Idle** - you haven't touched anything for a configurable duration (default: 30 seconds)
- **Alert** - you've been active for a configurable duration without a break (default: 1 hour) - time to get up!
- **Break** - you've stopped activity after an alert; the daemon waits for a break duration (default: 5 minutes) before returning to Idle

The state machine works as follows:
- Active → Idle (after `idle_after` duration of inactivity)
- Active → Alert (after `alert_after` duration of continuous activity)
- Alert → Break (after `idle_after` duration of inactivity)
- Break → Idle (after `break_after` duration of inactivity)
- Break → Alert (if you resume activity during your break)

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
- `get-up-timer_icon`: a simplified display showing state indicators (green dot for Active, orange circle for Idle, "GET UP" text for Alert, blue square for Break)
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

The daemon will search for configuration files in the following order:
1. Path provided as command-line argument
2. `$XDG_CONFIG_HOME/get-up-timer/config.toml`
3. `/etc/get-up-timer/config.toml`

If no configuration file is found, default values are used.

An example configuration is provided:

```toml
# Timing configuration
# Supports suffixes: s (seconds), m (minutes), h (hours)
[timing]
# After this duration of continuous activity show "GET UP" alert
alert_after = "1h"

# While in Alert state:
#  - after this duration of inactivity (idle), transition to Break state
break_after = "5m"

# After this duration of inactivity (no input), transition from Active to Idle state
#  NOTE: If you are usually looking at the screen thinking or pondering the universe,
#   you might need to increase this timer to avoid going into `IDLE` while you are 
#   still sitting down. (:
idle_after = "20s"

# Notifications configuration
[notifications]
# Whether sound notifications are enabled
sound_enabled = true

# Path to sound file to play when alert triggers (if sound_enabled = true)
sound_path = "/usr/share/sounds/alert.wav"

# Whether desktop notifications are enabled (using notify-send)
desktop_notifications = true
```

**Default values:**
- `alert_after`: 1h
- `break_after`: 5m
- `idle_after`: 30s
- `sound_enabled`: true
- `desktop_notifications`: true

## Contributing

PRs welcome! See `TODO.md` for ideas.
