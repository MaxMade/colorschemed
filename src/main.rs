//! Update colorscheme upon changes of `org.freedesktop.appearance` DBus interface (`color-scheme`).

pub mod cli;
pub mod config;
pub mod dbus;
pub mod theme;

use std::{io::Write as _, process::ExitCode};

use clap::Parser;
use futures_util::stream::StreamExt;
use tokio::io::AsyncBufReadExt;
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
    let config = match Config::load(cli.config) {
        Ok(config) => config,
        Err(error) => {
            log::error!("Unable to parse configuration: {}", error);
            return ExitCode::FAILURE;
        }
    };

    // Connect to DBus
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

    // Showtime
    log::info!("Start listening for setting changes");
    while let Some(signal) = stream.next().await {
        let args = match signal.args() {
            Ok(stream) => stream,
            Err(err) => {
                log::warn!("Unable to parse signal change: {}", err);
                continue;
            }
        };

        // Try to parse signal
        let signal = DBusSignal::new(args.namespace, args.key, args.value);
        let theme = match ThemeMode::try_from(signal) {
            Ok(theme) => theme,
            Err(err @ ThemeModeError::InvalidNameSpace(_))
            | Err(err @ ThemeModeError::InvalidKey(_)) => {
                log::warn!("Unable to parse signal change: {}", err);
                continue;
            }
            Err(err @ ThemeModeError::InvalidValue) => {
                log::error!("Unable to parse signal change: {}", err);
                return ExitCode::FAILURE;
            }
        };

        // Detected colorscheme change
        log::info!("New colorscheme change: {}", theme);

        'outer: for command in config.commands(theme) {
            // Ask user before executing (if necessary)
            'inner: while config.ask_before() {
                print!("Want to execute command \"{}\"? [yn] ", command);
                std::io::stdout().flush().unwrap();

                let stdin = tokio::io::stdin();
                let mut reader = tokio::io::BufReader::new(stdin);
                let mut input = String::new();

                reader.read_line(&mut input).await.unwrap();
                let ch = input.chars().next();
                match ch {
                    Some(c) if c == 'y' => break 'inner,
                    Some(c) if c == 'n' => continue 'outer,
                    Some(c) => println!("Invalid input {}, expected 'y' or 'no'", c),
                    None => println!("No input received"),
                }
            }

            log::debug!("Executing \"{}\"", command);

            // Expand variables
            let command = match shellexpand::env(command.as_str()) {
                Ok(command) => command,
                Err(error) => {
                    log::error!("Unable to expand environment variables: {}", error);
                    return ExitCode::FAILURE;
                }
            };

            // Split command
            let parts = match shell_words::split(command.as_ref()) {
                Ok(parts) => parts,
                Err(error) => {
                    log::error!("Unable to tokenize input command: {}", error);
                    return ExitCode::FAILURE;
                }
            };

            let (program, args) = match parts.split_first() {
                Some(parts) => parts,
                None => {
                    log::error!("Unable to tokenize empty command");
                    return ExitCode::FAILURE;
                }
            };

            // Spawn process
            let process = match tokio::process::Command::new(program)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(process) => process,
                Err(error) => {
                    log::error!("Unable to spawn command: {}", error);
                    return ExitCode::FAILURE;
                }
            };

            // Wait for process
            let output = match process.wait_with_output().await {
                Ok(output) => output,
                Err(error) => {
                    log::error!("Unable to wait for process: {}", error);
                    return ExitCode::FAILURE;
                }
            };

            if !output.status.success() {
                log::error!(
                    "Execution of command failed: {}",
                    String::from_utf8(output.stderr).unwrap()
                );
            }
        }
    }

    ExitCode::SUCCESS
}
