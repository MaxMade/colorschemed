# colorschemed

`colorschemed` is a small daemon that listens for system color scheme changes (light/dark mode) and executes user-defined commands in response. This makes it easy to automatically update application themes and configurations whenever the system theme changes.

## Features

* Reacts to light/dark mode switches
* Executes arbitrary commands on theme change
* Integrates with desktop environments via DBus
* Simple and extensible configuration (TOML)

## How it works

`colorschemed` listens to DBus signals.

When a color scheme change is detected, the configured commands are executed.

## Installation

Clone the repository and install using Cargo:

```
git clone https://github.com/MaxMade/colorschemed.git
cd colorschemed
cargo install --path=.
```

Set up the systemd user service:

```
cp systemd/colorschemed.service ~/.config/systemd/user/colorschemed.service
systemctl --user daemon-reload
systemctl start --user colorschemed
```

(Optional) Enable it to start automatically:

```
systemctl enable --user colorschemed
```

## Configuration

An example configuration is provided at:

```
config/example.toml
```

This includes sample commands for updating configurations of:

* Alacritty
* Helix
* Neovim
* Vim

Copy and modify it to suit your setup:

```
mkdir -p ~/.config/colorschemed
cp config/example.toml ~/.config/colorschemed/config.toml
```

Edit the file and define commands for light and dark modes.

## Notes

* Make sure your desktop environment supports the XDG portal settings interface.
* Commands should be idempotent and fast to avoid delays on theme switching.
* If commands do not run, check logs via `journalctl`.

## License

See LICENSE file for details.
