use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration, loaded from ~/.config/macterm/config.toml
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Scrollback buffer size (rows stored in history)
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,

    /// Number of panes to open on startup (can be overridden by CLI)
    #[serde(default = "default_panes")]
    pub default_panes: usize,

    /// Shell to use (e.g. "/bin/zsh", "/bin/bash")
    /// None = use system default shell
    pub shell: Option<String>,

    /// Custom keybindings (action -> key combo string)
    #[serde(default)]
    pub keybindings: std::collections::HashMap<String, String>,
}

fn default_scrollback() -> usize {
    10000
}

fn default_panes() -> usize {
    1
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback(),
            default_panes: default_panes(),
            shell: None,
            keybindings: std::collections::HashMap::new(),
        }
    }
}

impl Config {
    /// Load config from the default path (~/.config/macterm/config.toml).
    /// Returns Default if file doesn't exist or can't be parsed.
    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to read config file {:?}: {}", path, e);
                return Self::default();
            }
        };
        match toml::from_str(&content) {
            Ok(cfg) => {
                log::info!("Loaded config from {:?}", path);
                cfg
            }
            Err(e) => {
                log::warn!("Failed to parse config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    /// Path to the config file.
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("macterm");
        p.push("config.toml");
        p
    }
}
