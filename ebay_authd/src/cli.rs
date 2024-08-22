use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub enum Cli {
    /// Daemon control commands
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Testing commands
    Test {
        #[command(subcommand)]
        command: TestCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    /// Start the daemon
    Start,
    /// Get the status of the daemon
    Status,
    /// Fix a broken daemon instance
    Reauth,
    /// Ask the daemon to stop
    Stop,
}

#[derive(Debug, Subcommand)]
pub enum TestCommand {
    /// Get the latest token
    Token,
    /// Get the status of the daemon
    Status,
}
