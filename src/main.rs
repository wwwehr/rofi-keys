use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use clap::Parser;
use serde::{Deserialize, Serialize};

/// A keyboard-driven application launcher using Rofi
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Specify an alternate config file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Initialize a default config file and exit
    #[arg(long)]
    init: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    theme: Option<String>,
    menu_title: Option<String>,
    entries: Vec<MenuEntryConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MenuEntryConfig {
    key: String,
    label: String,
    command: String,
}

#[derive(Debug)]
struct MenuEntry {
    key: char,
    label: String,
    command: String,
}

#[derive(Debug)]
struct Menu {
    title: String,
    entries: Vec<MenuEntry>,
    theme: Option<String>,
}

impl Menu {
    fn new(title: &str, theme: Option<String>) -> Self {
        Menu {
            title: title.to_string(),
            entries: Vec::new(),
            theme,
        }
    }

    fn add_entry(&mut self, key: char, label: &str, command: &str) {
        self.entries.push(MenuEntry {
            key,
            label: label.to_string(),
            command: command.to_string(),
        });
    }

    fn generate_rofi_input(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("[{}] {}", entry.key, entry.label))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn get_command_for_key(&self, key: char) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.command.as_str())
    }

    fn display_with_rofi(&self) -> io::Result<Option<String>> {
        // Prepare key bindings for each menu entry
        let mut kb_args = Vec::new();
        let mut key_to_index: HashMap<char, i32> = HashMap::new();
        
        // For each entry, create a custom keybinding
        for (i, entry) in self.entries.iter().enumerate() {
            let kb_index = i + 1; // Rofi uses 1-based indexing for kb-custom
            kb_args.push(format!("-kb-custom-{}", kb_index));
            kb_args.push(entry.key.to_string());
            key_to_index.insert(entry.key, kb_index as i32);
        }
        
        // Generate menu items
        let menu_input = self.generate_rofi_input();
        
        // Basic Rofi arguments
        let mut rofi_args = vec![
            "-dmenu", 
            "-i", 
            "-p", 
            &self.title,
            "-no-fork",  // Added to prevent forking which may trigger systemd
            "-markup-rows",
            "-no-custom", // Disable manual entry
            "-theme-str", "configuration { matching: \"regex\"; }" // Use regex matching to avoid filtering
        ];
        
        // Add theme if specified
        if let Some(theme) = &self.theme {
            rofi_args.push("-theme");
            rofi_args.push(theme);
        }
        
        // Add all the key binding arguments
        for arg in kb_args.iter() {
            rofi_args.push(arg);
        }
        
        // Prepare and execute rofi command
        let mut child = Command::new("rofi")
            .args(rofi_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
            
        // Write menu items to rofi's stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(menu_input.as_bytes())?;
        }
        
        // Get rofi's output and exit status
        let output = child.wait_with_output()?;
        let exit_code = output.status.code().unwrap_or(0);
        
        // Check for direct key activation (custom-N exit codes)
        if exit_code >= 10 {
            // Find which key was pressed based on exit code
            let kb_index = exit_code - 9; // Custom-1 = 10, Custom-2 = 11, etc.
            
            // Find the key that corresponds to this index
            for (key, idx) in &key_to_index {
                if *idx == kb_index {
                    // Get the command for this key
                    if let Some(cmd) = self.get_command_for_key(*key) {
                        return Ok(Some(cmd.to_string()));
                    }
                }
            }
        }
        
        // If no direct key was detected, return None
        Ok(None)
    }
}

// Modified function to execute a command from string
// This avoids systemd scope issues
fn execute_command(command: &str) -> io::Result<()> {
    // Use sh -c to launch the program
    // This bypasses some of the systemd scoping issues
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

// Expand ~ to home directory in paths
fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

// Create a default configuration
fn create_default_config() -> Config {
    Config {
        theme: None, // Use Rofi's default theme
        menu_title: Some("Applications".to_string()),
        entries: vec![
            MenuEntryConfig {
                key: "f".to_string(),
                label: "Firefox".to_string(),
                command: "firefox".to_string(),
            },
            MenuEntryConfig {
                key: "p".to_string(),
                label: "Firefox Private".to_string(),
                command: "firefox --private-window".to_string(),
            },
            MenuEntryConfig {
                key: "m".to_string(),
                label: "MPV".to_string(),
                command: "mpv".to_string(),
            },
            MenuEntryConfig {
                key: "v".to_string(),
                label: "MPV (clipboard)".to_string(),
                command: "mpv \"$(xclip -o)\"".to_string(),
            },
            MenuEntryConfig {
                key: "t".to_string(),
                label: "Terminal".to_string(),
                command: "x-terminal-emulator".to_string(),
            },
        ],
    }
}

// Write a config to a specific path
fn write_config(config: &Config, path: &PathBuf) -> io::Result<()> {
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    // Write the config
    fs::write(path, json)?;
    println!("Configuration written to {}", path.display());
    
    Ok(())
}

// Get the default config path
fn get_default_config_path() -> io::Result<PathBuf> {
    let home = env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    
    let mut path = PathBuf::from(home);
    path.push(".config/rofi-keys/config.json");
    
    Ok(path)
}

// Function to load menu entries from JSON config file
fn load_config(config_path: &PathBuf) -> io::Result<Config> {
    // Check if the config file exists
    if !config_path.exists() {
        // Create a default config
        let default_config = create_default_config();
        
        // Write the default config
        write_config(&default_config, config_path)?;
        
        return Ok(default_config);
    }
    
    // Read and parse the JSON config
    let content = fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON config: {}", e)))?;
    
    Ok(config)
}

fn main() -> io::Result<()> {
    // Parse command-line arguments using Clap
    let cli = Cli::parse();
    
    // Get the config path (custom or default)
    let config_path = match cli.config {
        Some(path) => path,
        None => get_default_config_path()?,
    };
    
    // If --init flag is set, just create the config and exit
    if cli.init {
        let default_config = create_default_config();
        write_config(&default_config, &config_path)?;
        println!("Default configuration initialized at {}", config_path.display());
        return Ok(());
    }
    
    // Load configuration
    let config = match load_config(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config from {}: {}", config_path.display(), e);
            // Use default config if loading fails
            create_default_config()
        }
    };
    
    // Expand theme path if it exists
    let theme = config.theme.map(|t| expand_path(&t));
    
    // Create menu
    let mut menu = Menu::new(
        config.menu_title.as_deref().unwrap_or("Shortcuts"),
        theme,
    );
    
    // Add entries from config
    for entry in config.entries {
        if let Some(key_char) = entry.key.chars().next() {
            menu.add_entry(key_char, &entry.label, &entry.command);
        }
    }
    
    // Handle keyboard shortcut detection
    if let Some(command) = menu.display_with_rofi()? {
        execute_command(&command)?;
    }
    
    Ok(())
}
