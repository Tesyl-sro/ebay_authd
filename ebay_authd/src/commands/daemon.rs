use crate::{
    client::Client,
    config::configuration::Configuration,
    error::{Error, Result},
    tokenmgr::TokenManager,
};
use ebay_authd_core::{request::Request, response::Response};
use log::{debug, error, info, warn};
use nix::sys::{
    select::{select, FdSet},
    time::TimeVal,
};
use oauth2::{
    basic::BasicClient, reqwest::http_client, url::Url, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use std::{
    fs,
    io::stdin,
    os::{
        fd::{AsFd, AsRawFd, BorrowedFd},
        unix::net::UnixListener,
    },
    path::Path,
    str::FromStr,
};

pub const SOCKET_PATH: &str = "/tmp/ebay_authd.sock";
const TOKEN_URL: &str = "https://api.ebay.com/identity/v1/oauth2/token";
const AUTH_URL: &str = "https://auth.ebay.com/oauth2/authorize";
const REDIRECT_URL: &str =
    "https://signin.ebay.com/ws/eBayISAPI.dll?ThirdPartyAuthSucessFailure&isAuthSuccessful=true";
const SCOPES: [&str; 12] = [
    "https://api.ebay.com/oauth/api_scope",
    "https://api.ebay.com/oauth/api_scope/sell.marketing.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.inventory.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.account.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.fulfillment.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.analytics.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.finances",
    "https://api.ebay.com/oauth/api_scope/sell.payment.dispute",
    "https://api.ebay.com/oauth/api_scope/commerce.identity.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.reputation.readonly",
    "https://api.ebay.com/oauth/api_scope/commerce.notification.subscription.readonly",
    "https://api.ebay.com/oauth/api_scope/sell.stores.readonly",
];

pub fn start(config: &Configuration) -> Result<()> {
    info!("Creating client");
    let client = BasicClient::new(
        ClientId::new(config.appid.to_string()),
        Some(ClientSecret::new(config.certid.to_string())),
        AuthUrl::new(AUTH_URL.to_string())?,
        Some(TokenUrl::new(TOKEN_URL.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.to_string())?);

    debug!("Generating PKCE challenge");
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    info!("Authorizing...");
    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in SCOPES {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }
    let (auth_url, _) = auth_request.set_pkce_challenge(pkce_challenge).url();

    info!("Generated auth URL");
    println!("\nPlease authenticate using the following URL:");
    println!("{auth_url}\n");

    debug!("Waiting for authentication");
    println!("Enter the URL after authentication:");

    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    let buffer = buffer.trim_end().to_string();

    let url = Url::from_str(&buffer)?;
    let auth_code = url
        .query_pairs()
        .find(|pair| pair.0 == "code")
        .map(|(_, value)| value)
        .unwrap();

    let token_result = client
        .exchange_code(AuthorizationCode::new(auth_code.into()))
        .set_pkce_verifier(pkce_verifier)
        .request(http_client)
        .map_err(|_| Error::TokenRequest)?;

    let tman = TokenManager::new(client, token_result);

    info!("Success, starting daemon");
    daemon_loop(tman)?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn daemon_loop(mut tman: TokenManager) -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        warn!("Found existing socket, removing it");
        fs::remove_file(SOCKET_PATH)?;
    }

    debug!("Starting UNIX socket");
    let listener = UnixListener::bind(SOCKET_PATH)?;
    listener.set_nonblocking(true)?;
    let mut clients: Vec<Client> = Vec::new();

    'outer: loop {
        let mut fds = FdSet::new();
        fds.insert(listener.as_fd());

        for client in &clients {
            let copy = unsafe { BorrowedFd::borrow_raw(client.as_raw_fd()) };
            fds.insert(copy);
        }

        let ready = select(
            None,
            Some(&mut fds),
            None,
            None,
            Some(&mut TimeVal::new(1, 0)),
        )?;

        if ready == 0 {
            tman.tick()?;
        }

        for fd in fds.fds(None) {
            if fd.as_raw_fd() == listener.as_raw_fd() {
                debug!("New client!");
                clients.push(Client::new(listener.accept()?.0)?);
                continue;
            }

            debug!("Handling client");

            let (index, client) = clients
                .iter_mut()
                .enumerate()
                .find(|(_, client)| **client == fd)
                .unwrap();

            let Some(message) = client.await_message()? else {
                warn!("Client broken, kicking");

                continue;
            };

            let Some(request) = message.into_request() else {
                error!("Malformed message: expected requset, got response");
                continue;
            };

            debug!("Handling client request");

            if let Err(why) = handle_client(client, request, &mut tman) {
                if matches!(why, Error::StopRequested) {
                    break 'outer;
                }

                error!("Failed to process request: {why}");
            }

            clients.remove(index);
        }
    }

    info!("Closing socket");
    fs::remove_file(SOCKET_PATH)?;

    Ok(())
}

fn handle_client(client: &mut Client, request: Request, tman: &mut TokenManager) -> Result<()> {
    match request {
        Request::Token => client.message(Response::Token(tman.get_token().into()))?,
        Request::Status => {
            client.message(Response::Status {
                version: env!("CARGO_PKG_VERSION").into(),
                expiry: tman.expiry(),
            })?;
        }
        Request::ForceRefresh => {
            tman.refresh()?;
        }
        Request::Stop => {
            info!("Stop requested");
            return Err(Error::StopRequested);
        }
    };

    Ok(())
}
