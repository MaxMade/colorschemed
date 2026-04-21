//! Parser configuration

use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::theme::ThemeMode;

const APP_NAME: &'static str = "colorschemed";

/// Errors that can occur while loading or parsing the configuration file.
#[derive(Debug)]
pub enum ConfigError {
    /// An error occurred while performing file I/O.
    IO(io::Error),

    /// An error occurred while parsing the TOML configuration.
    TOML(toml::de::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IO(error) => write!(f, "{}", error),
            ConfigError::TOML(error) => write!(f, "{}", error),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        ConfigError::IO(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::TOML(value)
    }
}

/// A placeholder value used for mapping a "light" and a "dark" variant.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placeholder {
    /// Value used in light mode or light theme contexts.
    light: String,

    /// Value used in dark mode or dark theme contexts.
    dark: String,
}

impl Placeholder {
    fn get(&self, mode: ThemeMode) -> &str {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }
}

/// Configuration for command execution.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commands {
    /// If true, the user will be prompted before executing commands.
    #[serde(rename = "ask-before")]
    ask_before_execute: bool,

    /// List of commands.
    #[serde(rename = "commands")]
    commands: Vec<String>,
}

impl Default for Commands {
    fn default() -> Self {
        Self {
            ask_before_execute: false,
            commands: Vec::new(),
        }
    }
}

/// Root configuration structure for the application.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Mapping of string keys to placeholder definitions.
    #[serde(rename = "Mappings")]
    mappings: HashMap<String, Placeholder>,

    /// Command execution configuration section.
    #[serde(rename = "Commands")]
    commands: Commands,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            mappings: HashMap::default(),
            commands: Commands::default(),
        };

        config.mappings.insert(
            "{{-mode-}}".to_string(),
            Placeholder {
                light: "light".to_string(),
                dark: "dark".to_string(),
            },
        );
        config.mappings.insert(
            "{{-Mode-}}".to_string(),
            Placeholder {
                light: "Light".to_string(),
                dark: "Dark".to_string(),
            },
        );
        config.mappings.insert(
            "{{-MODE-}}".to_string(),
            Placeholder {
                light: "LIGHT".to_string(),
                dark: "DARK".to_string(),
            },
        );

        config
    }
}

impl Config {
    /// Loads a configuration from a TOML file if provided. Otherwise fallback
    /// to default directories or, finally, default config.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::IO` if the file cannot be read, or
    /// `ConfigError::Toml` if parsing fails.
    pub fn load<P: AsRef<Path>>(provided: Option<P>) -> Result<Config, ConfigError> {
        let mut paths = Vec::new();

        // Add user-provided config file
        if let Some(provided) = provided {
            paths.push(PathBuf::from(provided.as_ref()));
        }

        // Add system-wide configuration directory
        if cfg!(unix) {
            paths.push(PathBuf::from("/etc").join(APP_NAME).join("config.toml"));
        } else if cfg!(windows) {
            paths.push(
                PathBuf::from(r"C:\ProgramData")
                    .join(APP_NAME)
                    .join("config.toml"),
            );
        }

        // Start with default config
        let mut config = Config::default();

        // Add local configuration directory
        if let Some(base_dirs) = BaseDirs::new() {
            paths.push(
                PathBuf::from(base_dirs.config_dir())
                    .join(APP_NAME)
                    .join("config.toml"),
            );
        }

        // Try directories
        for path in paths {
            log::trace!("Trying configuration \"{:?}\"", path);

            // Open file for reading
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(_) => {
                    // Ignoring invalid file...
                    continue;
                }
            };

            // Read the file contents
            let mut contents = String::new();
            if let Err(error) = file.read_to_string(&mut contents) {
                return Err(ConfigError::IO(error));
            }

            // Parse Configuration
            let c: Config = match toml::from_str(&contents) {
                Ok(c) => c,
                Err(error) => {
                    return Err(ConfigError::TOML(error));
                }
            };

            config = c;
            break;
        }

        let mappings = config
            .mappings
            .drain()
            .map(|(mut key, value)| {
                key.insert_str(0, "{{-");
                key.push_str("-}}");
                (key, value)
            })
            .collect();
        config.mappings = mappings;

        Ok(config)
    }

    /// Return a flag indicating the user should be prompted before executing commands.
    pub fn ask_before(&self) -> bool {
        self.commands.ask_before_execute
    }

    /// Get an [`Iterator`] over target commands with all mapping expanded.
    pub fn commands(&self, mode: ThemeMode) -> impl Iterator<Item = String> {
        self.commands.commands.iter().map(move |command| {
            let mut command = command.clone();

            for (identifier, placeholder) in self.mappings.iter() {
                let placeholder = placeholder.get(mode);
                command = command.replace(identifier, placeholder);
            }

            command
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let default = Config::default();

        let input = Config::load(Some("config/config.toml")).unwrap();

        assert!(input == default);
    }

    #[test]
    fn example_config() {
        let input = Config::load(Some("config/example.toml"));
        assert!(input.is_ok());
    }
}
