# 🎮 Rofi Keys

> ⌨️ A blazing-fast keyboard-driven application launcher using Rofi

## ✨ Features

- 🔤 **Single-key execution** - Press one key to launch apps
- 🛠️ **JSON configuration** - Easy to customize
- 🎨 **Theme support** - Use your favorite Rofi theme
- 📋 **Clipboard integration** - Play videos from clipboard with MPV
- 🚀 **No filtering mode** - Keys trigger actions directly

## 🔧 Installation

### Prerequisites

- Rust & Cargo
- Rofi
- MPV (optional)
- Firefox (optional)
- xclip (for clipboard support)

### Build from source

```bash
# Clone the repository
git clone https://github.com/yourusername/rofi-keys.git
cd rofi-keys

# Build with cargo
cargo build --release

# Install to your path (optional)
cp target/release/rofi-keys ~/.local/bin/
```

## 🚀 Quick Start

```bash
# Initialize a default config
rofi-keys --init

# Run it!
rofi-keys
```

## ⚙️ Configuration

The configuration is stored in JSON format at `~/.config/rofi-keys/config.json`:

```json
{
  "theme": null,
  "menu_title": "Applications",
  "entries": [
    {
      "key": "f",
      "label": "Firefox",
      "command": "firefox"
    },
    {
      "key": "p",
      "label": "Firefox Private",
      "command": "firefox --private-window"
    },
    {
      "key": "v",
      "label": "MPV (clipboard)",
      "command": "mpv \"$(xclip -o)\""
    },
    {
      "key": "t",
      "label": "Terminal",
      "command": "x-terminal-emulator"
    }
  ]
}
```

### 🎨 Custom Themes

You can specify a Rofi theme in the config:

```json
"theme": "~/.config/rofi/themes/custom.rasi"
```

## 🖥️ Usage

### Command Line Options

```
USAGE:
    rofi-keys [OPTIONS]

OPTIONS:
    -c, --config <FILE>    Specify an alternate config file path
    --init                 Initialize a default config file and exit
    -h, --help             Show help information
    -V, --version          Show version information
```

### 🔑 Key Bindings

The default configuration sets up:

- `f` - Firefox
- `p` - Firefox Private Browsing
- `v` - MPV with clipboard content
- `t` - Terminal

## 💡 Tips & Tricks

### 🎬 Playing Videos

```json
{
  "key": "y",
  "label": "YouTube (clipboard)",
  "command": "mpv --ytdl \"$(xclip -o)\""
}
```

### ⏰ Tea Timer

```json
{
  "key": "t",
  "label": "Tea Timer (4min)",
  "command": "kitty countdown 4m && notify-send 'Tea is ready! 🍵'"
}
```

### 🔊 Volume Control

```json
{
  "key": "m",
  "label": "Toggle Mute",
  "command": "pactl set-sink-mute @DEFAULT_SINK@ toggle"
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Rofi](https://github.com/davatorium/rofi) - The awesome application launcher
- [Rust](https://www.rust-lang.org/) - The programming language
- [Clap](https://github.com/clap-rs/clap) - Command Line Argument Parser for Rust
