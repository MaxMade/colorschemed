//! Command line interface

use std::path::PathBuf;

use clap::Parser;

/// Command line interface.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
