//! Theme-related information.

use std::{error::Error, fmt::Display};

use crate::dbus::DBusSignal;

const NAMESPACE: &'static str = "org.freedesktop.appearance";
const KEY: &'static str = "color-scheme";

/// [`Error`] related to processing theme changes.
#[derive(Debug)]
pub enum ThemeModeError<'a> {
    /// Invalid namespace.
    InvalidNameSpace(&'a str),
    /// Invalid key.
    InvalidKey(&'a str),
    /// Invalid value.
    InvalidValue,
}

impl<'a> Display for ThemeModeError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeModeError::InvalidNameSpace(namespace) => {
                write!(
                    f,
                    "Invalid Namespace: {} (expecting \"{}\")",
                    namespace, NAMESPACE
                )
            }
            ThemeModeError::InvalidKey(key) => {
                write!(f, "Invalid Key: {} (expecting \"{}\")", key, KEY)
            }
            ThemeModeError::InvalidValue => {
                write!(
                    f,
                    "Invalid Value (expecting \"{}/{}\")",
                    ThemeMode::Light as usize,
                    ThemeMode::Dark as usize
                )
            }
        }
    }
}

impl<'a> Error for ThemeModeError<'a> {
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

/// Parsed theme mode.
#[derive(Debug, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light theme.
    Light = 2,
    /// Dark theme.
    Dark = 1,
}

impl ThemeMode {}

impl Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::Light => write!(f, "Light"),
            ThemeMode::Dark => write!(f, "Dark"),
        }
    }
}

impl<'a> TryFrom<DBusSignal<'a>> for ThemeMode {
    type Error = ThemeModeError<'a>;

    fn try_from(value: DBusSignal<'a>) -> Result<Self, Self::Error> {
        match value.namespace() {
            NAMESPACE => match value.key() {
                KEY => {
                    let value = match value.value().downcast_ref() {
                        Ok(zvariant::Value::U8(value)) => value as usize,
                        Ok(zvariant::Value::Bool(value)) => value as usize,
                        Ok(zvariant::Value::I16(value)) => value as usize,
                        Ok(zvariant::Value::U16(value)) => value as usize,
                        Ok(zvariant::Value::I32(value)) => value as usize,
                        Ok(zvariant::Value::U32(value)) => value as usize,
                        Ok(zvariant::Value::I64(value)) => value as usize,
                        Ok(zvariant::Value::U64(value)) => value as usize,
                        _ => return Err(ThemeModeError::InvalidValue),
                    };

                    if value == ThemeMode::Light as usize {
                        Ok(ThemeMode::Light)
                    } else if value == ThemeMode::Dark as usize {
                        Ok(ThemeMode::Dark)
                    } else {
                        todo!();
                    }
                }
                key => Err(ThemeModeError::InvalidKey(key)),
            },
            namespace => Err(ThemeModeError::InvalidNameSpace(namespace)),
        }
    }
}
