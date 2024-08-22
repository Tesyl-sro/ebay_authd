use super::daemon::SOCKET_PATH;
use crate::{
    client::Client,
    error::{Error, Result},
};
use ebay_authd_core::{request::Request, response::Response};
use log::debug;
use std::os::unix::net::UnixStream;

pub fn token() -> Result<()> {
    let mut client = send_msg_to_server(Request::Token)?;
    let response = client
        .await_message()?
        .unwrap()
        .into_response()
        .ok_or(Error::ExpectedResponse)?;

    let Response::Token(token) = response else {
        return Err(Error::UnexpectedResponse);
    };

    println!("{token}");
    Ok(())
}

pub fn status() -> Result<()> {
    let mut client = send_msg_to_server(Request::Status)?;
    let response = client
        .await_message()?
        .unwrap()
        .into_response()
        .ok_or(Error::ExpectedResponse)?;

    let Response::Status { version, expiry } = response else {
        return Err(Error::UnexpectedResponse);
    };

    println!("Deamon is active and running.");
    println!("Version: {version:?}");
    println!("Next token in: {}s", expiry.as_secs());

    Ok(())
}

pub fn reauth() -> Result<()> {
    send_msg_to_server(Request::ForceRefresh)?;
    Ok(())
}

pub fn stop() -> Result<()> {
    send_msg_to_server(Request::Stop)?;
    Ok(())
}

fn send_msg_to_server(req: Request) -> Result<Client> {
    debug!("Sending request: {req:?}");

    let client = UnixStream::connect(SOCKET_PATH)?;
    let mut client = Client::new(client)?;

    client.message(req)?;

    Ok(client)
}
