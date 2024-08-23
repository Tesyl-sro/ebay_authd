use super::daemon::SOCKET_PATH;
use crate::error::{Error, Result};
use colored::Colorize;
use ebay_authd_client::Client;
use ebay_authd_core::{request::Request, response::Response};
use std::os::unix::net::UnixStream;

pub fn token() -> Result<()> {
    let mut client = connect()?;
    let response = client.exchange(Request::Token)?;

    let Response::Token(token) = response else {
        return Err(Error::UnexpectedResponse);
    };

    println!("{token}");
    Ok(())
}

pub fn status() {
    let mut client = match connect() {
        Ok(client) => client,
        Err(why) => {
            eprintln!("{} {why}", "Failed to connect to daemon:".red());
            return;
        }
    };

    let response = match client.exchange(Request::Status) {
        Ok(response) => response,
        Err(why) => {
            eprintln!("{} {why}", "Failed to communicate to daemon:".red());
            return;
        }
    };

    let Response::Status {
        version,
        expiry,
        last_refresh,
        short_token,
        short_refresh_token,
    } = response
    else {
        eprintln!("{} {response:?}", "Daemon sent wrong response:".red());
        return;
    };

    println!("Daemon: {}", "Running".green());
    println!("Version: {}", version.blue());
    println!(
        "Token expiry: {}{}",
        expiry.as_secs().to_string().yellow(),
        "s".yellow()
    );
    println!(
        "Last refresh: {}{} {}",
        last_refresh.as_secs().to_string().yellow(),
        "s".yellow(),
        "ago".blue()
    );

    println!(
        "Current token: {}{}",
        short_token.bright_cyan(),
        "...".bright_red()
    );
    println!(
        "Refresh token: {}{}",
        short_refresh_token.bright_cyan(),
        "...".bright_red()
    );
}

pub fn reauth() -> Result<()> {
    connect()?.exchange(Request::ForceRefresh)?;
    Ok(())
}

pub fn stop() -> Result<()> {
    connect()?.exchange(Request::Stop)?;
    Ok(())
}

fn connect() -> Result<Client> {
    let client = UnixStream::connect(SOCKET_PATH)?;
    let client = Client::new(client)?;

    Ok(client)
}
