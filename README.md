# Whatawhat
## A tool for monitoring activity on the computer throughout the day.

### Why would I want this?
- Because it's hard to remember what you've been doing 3 hours ago, let alone 1 week ago.
- Because having no idea where your time goes will worsen your self-esteem.
- Because knowing how much you spend on games might help you cope with addictions.


### Introducing Whatawhat
A simple cli all-in-one tool for monitoring activity. The only tool you need is whatawhat cli and you're set.

**No runtime required.** No python, no node. The application is a single executable that takes up 1MB during execution.

**Cli friendly.** Very easy to use with tools like `grep` and `less`.

**Everything is local.** As long as your computer is safe, your data is safe.

### Installation

#### Windows
```bash
cargo install -F win whatawhat
```


#### X11 Linux
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


### Examples

Get application usage for this week.
```
whatawhat.exe timeline -d 1 -o days --start "last monday"
```

View the timeline for last 8 hours.
```
whatawhat.exe timeline -d 30 -o minutes --start "8 hours ago"
```

**Whatawhat also works well with common cli utils like grep**

View when you started and ended your day with head/tail
```
whatawhat.exe timeline -d 1 -o minutes --start "yesterday" --end  "yesterday" --days | tail
```
*or*
```
whatawhat.exe timeline -d 1 -o minutes --start "yesterday" --end  "yesterday" --days | head
```

View what YouTube videos you've been watching with grep
```
whatawhat.exe timeline -d 1 -o minutes --start "today" --days | grep YouTube
```

### Notes

Whatawhat doesn't run startup by default. This needs to be configured yourself.

For Windows you can refer to [this][https://www.howtogeek.com/208224/how-to-add-a-program-to-startup-in-windows/]:
    - Create a bat file with `whatawhat init` in startup directory ("%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup).
For Linux you can refer to [this][https://askubuntu.com/questions/814/how-to-run-scripts-on-start-up].
    - Create a bat file with `whatawhat init` in startup directory ("%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup).
    - crontab -e


