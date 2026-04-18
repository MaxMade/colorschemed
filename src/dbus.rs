//! DBus Interface

use std::fmt::Display;

use zbus::proxy;
use zvariant::OwnedValue;

/// Proxy for `org.freedesktop.appearance` interface (service: `org.freedesktop.portal.Desktop`).
#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait Settings {
    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: OwnedValue) -> zbus::Result<()>;
}

/// DBus Signal.
#[derive(Debug)]
pub struct DBusSignal<'a> {
    namespace: &'a str,
    key: &'a str,
    value: OwnedValue,
}

impl<'a> DBusSignal<'a> {
    /// Create new [`DBusSignal`].
    pub fn new(namespace: &'a str, key: &'a str, value: OwnedValue) -> Self {
        Self {
            namespace,
            key,
            value,
        }
    }

    /// Get namespace.
    pub fn namespace(&self) -> &'a str {
        self.namespace
    }

    /// Get key.
    pub fn key(&self) -> &'a str {
        self.key
    }

    /// Get value.
    pub fn value(&self) -> &OwnedValue {
        &self.value
    }
}

impl<'a> Display for DBusSignal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.value {
            zvariant::Value::U8(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::Bool(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::I16(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::U16(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::I32(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::U32(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::I64(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::U64(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::F64(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::Str(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
            zvariant::Value::Signature(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::ObjectPath(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::Value(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::Array(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::Dict(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::Structure(value) => {
                write!(f, "{}.{} = {}", self.namespace, self.key, value)
            }
            zvariant::Value::Fd(value) => write!(f, "{}.{} = {}", self.namespace, self.key, value),
        }
    }
}
