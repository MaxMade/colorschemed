//! Update colorscheme upon changes of `org.freedesktop.appearance` DBus interface (`color-scheme`).

pub mod cli;
pub mod config;
pub mod dbus;
pub mod theme;

use std::process::ExitCode;

use clap::Parser;
use futures_util::stream::StreamExt;
use tokio::{select, signal};
use zbus::Connection;

use crate::{
    cli::Cli,
    config::Config,
    dbus::{DBusSignal, SettingsProxy},
    theme::{ThemeMode, ThemeModeError},
};

#[tokio::main]
async fn main() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialise environment logger
    let verbose = cli.verbose;
    let mut builder = env_logger::Builder::from_default_env();
    if verbose == 0 {
        builder.init();
    } else if verbose == 1 {
        builder.filter(None, log::LevelFilter::Info).init();
    } else if verbose == 2 {
        builder.filter(None, log::LevelFilter::Debug).init();
    } else {
        builder.filter(None, log::LevelFilter::Trace).init();
    }

    // Parse configuration
    let config = match cli.config {
        Some(path) => match Config::new(path) {
            Ok(config) => config,
            Err(error) => {
                log::error!("Unable to parse configuration: {}", error);
                return ExitCode::FAILURE;
            }
        },
        None => Config::default(),
    };

    let conn = match Connection::session().await {
        Ok(conn) => conn,
        Err(err) => {
            log::error!("Unable to connect to session bus: {}", err);
            return ExitCode::FAILURE;
        }
    };
    let proxy = match SettingsProxy::new(&conn).await {
        Ok(proxy) => proxy,
        Err(err) => {
            log::error!("Unable to create proxy: {}", err);
            return ExitCode::FAILURE;
        }
    };
    let mut stream = match proxy.receive_setting_changed().await {
        Ok(stream) => stream,
        Err(err) => {
            log::error!("Unable to register for changed settings signals: {}", err);
            return ExitCode::FAILURE;
        }
    };

    log::info!("Start listening for setting changes");

    loop {
        select! {
            _ = signal::ctrl_c() => {
                log::debug!("Received Ctrl+C, exiting...");
                break;
            }
            Some(signal) = stream.next() => {
                let args = match signal.args() {
                    Ok(stream) => stream,
                    Err(err) => {
                        log::warn!("Unable to parse signal change: {}", err);
                        continue;
                    }
                };
                let signal = DBusSignal::new(args.namespace, args.key, args.value);
                let theme = match ThemeMode::try_from(signal) {
                    Ok(theme) => theme,
                    Err(err @ ThemeModeError::InvalidNameSpace(_)) | Err(err @ ThemeModeError::InvalidKey(_)) => {
                        log::warn!("Unable to parse signal change: {}", err);
                        continue;
                    }
                    Err(err @ ThemeModeError::InvalidValue) => {
                        log::error!("Unable to parse signal change: {}", err);
                        return ExitCode::FAILURE;
                    }
                };

                log::info!("New colorscheme change: {}", theme);
            }
        }
    }

    ExitCode::SUCCESS
}
