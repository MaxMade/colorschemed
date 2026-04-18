//! Update colorscheme upon changes of `org.freedesktop.appearance` DBus interface (`color-scheme`).

pub mod cli;
pub mod dbus;
pub mod theme;

use std::process::ExitCode;

use clap::Parser;
use futures_util::stream::StreamExt;
use tokio::{select, signal};
use zbus::Connection;

use crate::{
    cli::Cli,
    dbus::{DBusSignal, SettingsProxy},
    theme::{ThemeMode, ThemeModeError},
};

#[tokio::main]
async fn main() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    let conn = match Connection::session().await {
        Ok(conn) => conn,
        Err(_) => todo!(),
    };
    let proxy = match SettingsProxy::new(&conn).await {
        Ok(proxy) => proxy,
        Err(_) => todo!(),
    };
    let mut stream = match proxy.receive_setting_changed().await {
        Ok(stream) => stream,
        Err(_) => todo!(),
    };

    println!("Start listening for setting changes");

    loop {
        select! {
            _ = signal::ctrl_c() => {
                println!("Received Ctrl+C, exiting...");
                break;
            }
            Some(signal) = stream.next() => {
                let args = match signal.args() {
                    Ok(stream) => stream,
                    Err(_) => todo!(),
                };
                let signal = DBusSignal::new(args.namespace, args.key, args.value);
                let theme = match ThemeMode::try_from(signal) {
                    Ok(theme) => theme,
                    Err(err @ ThemeModeError::InvalidNameSpace(_)) | Err(err @ ThemeModeError::InvalidKey(_)) => {
                        eprintln!("Unable to parse signal change: {}", err);
                        continue;
                    }
                    Err(err @ ThemeModeError::InvalidValue) => {
                        eprintln!("Unable to parse signal change: {}", err);
                        return ExitCode::FAILURE;
                    }
                };

                println!("New colorscheme change: {}", theme);
            }
        }
    }

    ExitCode::SUCCESS
}
