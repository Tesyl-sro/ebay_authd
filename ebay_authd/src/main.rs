#![allow(
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc
)]

use crate::error::Result;
use clap::Parser;
use cli::{Cli, DaemonCommand, TestCommand};
use config::configuration::Configuration;
use log::{debug, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::process::exit;

mod cli;
mod client;
mod commands;
mod config;
mod error;
pub mod tokenmgr;

fn main() -> Result<()> {
    let mut logger = SimpleLogger::new()
        .with_module_level("reqwest", LevelFilter::Off)
        .with_module_level("rustls", LevelFilter::Off)
        .with_level(LevelFilter::Info);

    if cfg!(debug_assertions) {
        logger = logger.with_level(LevelFilter::Debug);
    }

    logger.init().unwrap();

    let cli = Cli::parse();

    if !config::config_exists()? {
        info!("Creating default empty configuration");
        config::create_config()?;
        exit(0);
    }

    debug!("Loading configuration");
    let config: Configuration = confy::load_path(config::location()?)?;

    match cli {
        Cli::Daemon { command } => match command {
            DaemonCommand::Start => commands::daemon::start(&config)?,
            DaemonCommand::Reauth => commands::testcmds::reauth()?,
            DaemonCommand::Stop => commands::testcmds::stop()?,
            DaemonCommand::Status => commands::testcmds::status()?,
        },
        Cli::Test { command } => match command {
            TestCommand::Token => commands::testcmds::token()?,
        },
    }

    Ok(())
}
