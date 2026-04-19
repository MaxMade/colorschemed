//! Parser configuration

use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::File,
    io::{self, Read},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// Errors that can occur while loading or parsing the configuration file.
#[derive(Debug)]
pub enum ConfigError {
    /// An error occurred while performing file I/O.
    IO(io::Error),

    /// An error occurred while parsing the TOML configuration.
    Toml(toml::de::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IO(error) => write!(f, "{}", error),
            ConfigError::Toml(error) => write!(f, "{}", error),
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
        ConfigError::Toml(value)
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
            "mode".to_string(),
            Placeholder {
                light: "light".to_string(),
                dark: "dark".to_string(),
            },
        );
        config.mappings.insert(
            "Mode".to_string(),
            Placeholder {
                light: "Light".to_string(),
                dark: "Dark".to_string(),
            },
        );
        config.mappings.insert(
            "MODE".to_string(),
            Placeholder {
                light: "LIGHT".to_string(),
                dark: "DARK".to_string(),
            },
        );

        config
    }
}

impl Config {
    /// Loads a configuration from a TOML file at the given path.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::IO` if the file cannot be read, or
    /// `ConfigError::Toml` if parsing fails.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut file = File::open(path.as_ref())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let config: Config = toml::from_str(content.as_str())?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let default = Config::default();

        let input = Config::new("config/config.toml").unwrap();

        assert!(input == default);
    }

    #[test]
    fn example_config() {
        let input = Config::new("config/example.toml");
        assert!(input.is_ok());
    }
}
