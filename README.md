# Whatawhat
A tool for monitoring activity on the computer throughout the day.

![Whatawhat demo](assets/whatawhat-demo.gif)

- [Why would I want this?](#why-would-i-want-this)
- [Introduction](#introducing-whatawhat)
- [Installation](#installation)
  - [Windows](#windows)
  - [X11 Linux](#x11-linux)
- [Usage](#usage)
- [Examples](#examples)
- [Autostart](#autostart)
- [Notes](#installation)
- [Future](#future)

## Why would I want this?
- Because it's hard to remember what you've been doing 3 hours ago, let alone 1 week ago.
- Because having no idea where your time goes will worsen your self-esteem.
- Because knowing how much you spend on games might help you cope with addictions.



## Introducing Whatawhat
A simple cli/daemon for monitoring your activity.

**No runtime required.** No python, no node. The application is a single executable that takes up 1MB during execution.

**Cli friendly.** Very easy to use with tools like `grep` and `less`.

**Everything is local.** You control your data. your data does not leave your computer, not used by advertisers, not leaked by corporations.

## Installation

### Windows
```bash
cargo install -F win whatawhat
```


### X11 Linux
To compile and run the application you need xcb and xscreensaver.
Some distros (like Manjaro) will have them preinstalled.

To get utilities on Ubuntu you can simply run:
```bash
sudo apt-get install libxcb1-dev
sudo apt-get install xscreensaver
```
Then use cargo install to build the program:
```bash
cargo install -F x11 whatawhat
```

## Usage
When you're first starting out it's recommended to run `whatawhat restart`. This will start the daemon for the current session.

Now you can use the `whatawhat timeline` to get different data about your activity.

For details on how to run the deamon on boot refer to [Autostart](#autostart)

## Examples

Get application usage for this week:
```
whatawhat timeline -d 1 -o days --start "last monday"
```

View the timeline for last 8 hours:
```
whatawhat timeline -d 30 -o minutes --start "8 hours ago"
```

**Whatawhat also works well with common cli utils like grep**

View when you started and ended your day with head/tail (assuming you weren't working at night):
```
whatawhat timeline -d 1 -o minutes --start "yesterday" --end  "yesterday" --days | tail
```
*or*
```
whatawhat timeline -d 1 -o minutes --start "yesterday" --end  "yesterday" --days | head
```

View what YouTube videos you've been watching with grep:
```
whatawhat timeline -d 1 -o minutes --start "today" --days | grep YouTube
```

## Autostart

Whatawhat doesn't run startup by default. This needs to be configured yourself.

For Windows you can refer to [this](https://www.howtogeek.com/208224/how-to-add-a-program-to-startup-in-windows/):
 - Create a shortcut to whatawhat-daemon.exe.
 - Put the shortcut into the startup folder.
 - The daemon will now autostart on boot.

On Linux it's best to use autostart utilities provided by Gnome, KDE Plasma, etc.:
 - Add a new process on startup.
 - Specify the full path to the daemon (Usually `/home/username/.cargo/bin/whatawhat-daemon`).
 - The daemon will now autostart on boot.

## Notes

1. Dates are formatted using [chrono-english](https://github.com/stevedonovan/chrono-english). The supported formats are:
    - Relative dates: "today", "yesterday", "last monday", "1 week/day/hour/minute ago".
    - Normal dates "00:00 28/03/2025", "3pm 28/03/2025" or just "28/03/2025" (If you want to use different Us style dates add `--date-style us` flag).
    - Combined: "10:00 1 week ago".

1. By default `whatawhat timeline` will trim items which have less than 1% of the total time of usage. You can set `--percentage 0%` to change this.

1. If you want to go through entire days, you can use `--days` flag. For example if you want to get data for yesterday you can use `--start "yesterday" --end "yesterday" --days`. This will show data from start of "yesterday" to the end of "yesterday".


## Future
- **Add configuration files**. Things like date formatting, collection interval should probably be modifiable. Also, defaults for `whatawhat timeline` should be configurable.
- **Colors**. Colors can be extracted from icons for applications and will make it easier to distinguish between items.
- **More tests**. The application lacks in integration tests and units tests for some components.
