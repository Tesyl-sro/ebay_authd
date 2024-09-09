use crate::{
    config::configuration::Configuration,
    error::{Error, Result},
    tokenmgr::TokenManager,
};
use ebay_authd_client::Client;
use ebay_authd_core::{request::Request, response::Response};
use log::{debug, error, info, warn};
use nix::{
    errno::Errno,
    sys::{
        select::{select, FdSet},
        time::TimeVal,
    },
};
use oauth2::{
    basic::BasicClient, reqwest::http_client, url::Url, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use std::{
    env, fs,
    io::stdin,
    os::{
        fd::{AsFd, AsRawFd, BorrowedFd},
        unix::net::UnixListener,
    },
    path::Path,
    process::{exit, Command},
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
    thread::sleep,
    time::Duration,
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

static STOP: AtomicBool = AtomicBool::new(false);

pub fn start(config: &Configuration, screen: bool) -> Result<()> {
    if screen {
        check_screen()?;

        if !running_in_screen() {
            info!("Restarting using screen");
            restart_in_screen()?;
        }

        info!("Screen session detected");
    }

    info!("Creating clie7nt");
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

    if screen {
        info!("Detaching screen");
        detach_screen()?;
    }

    daemon_loop(tman)?;
    info!("Daemon stopped");

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

    ctrlc::set_handler(|| {
        STOP.store(true, Ordering::SeqCst);
        info!("Please wait...");
    })
    .unwrap();

    'outer: loop {
        if STOP.load(Ordering::Relaxed) {
            info!("Got stop signal");
            break;
        }

        let mut fds = FdSet::new();
        fds.insert(listener.as_fd());

        for client in &clients {
            let copy = unsafe { BorrowedFd::borrow_raw(client.as_raw_fd()) };
            fds.insert(copy);
        }

        let select_result = select(
            None,
            Some(&mut fds),
            None,
            None,
            Some(&mut TimeVal::new(1, 0)),
        );

        match select_result {
            Ok(0) => tman.tick()?,
            Err(Errno::EINTR) => continue,
            Err(other) => return Err(other.into()),
            Ok(..) => (),
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

            let message = match client.await_message() {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    warn!("Client broken, kicking");
                    clients.remove(index);
                    continue;
                }
                Err(why) => {
                    error!("Failed to parse message: {why}");
                    clients.remove(index);
                    continue;
                }
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
                last_refresh: tman.last_refresh(),
                short_token: tman.short_token(),
                short_refresh_token: tman.short_refresh_token(),
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

fn check_screen() -> Result<()> {
    match Command::new("screen").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version_string = String::from_utf8(output.stdout)?;
            debug!("Found screen: {version_string}");

            Ok(())
        }
        Ok(output) if !output.status.success() => {
            error!(
                "screen found but got unexpected status code: {}",
                output.status
            );
            Err(Error::Screen)
        }
        Ok(..) => unreachable!(),
        Err(why) => {
            error!("screen not installed or not in PATH: {why}");
            Err(Error::Screen)
        }
    }
}

fn restart_in_screen() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Create a new session
    Command::new("screen")
        .arg("-dmS")
        .arg("ebay_authd")
        .args(args)
        .output()?;

    sleep(Duration::from_secs(2));

    // Attach to the new session
    Command::new("screen")
        .arg("-r")
        .arg("ebay_authd")
        .spawn()?
        .wait()?;

    exit(0);
}

fn running_in_screen() -> bool {
    env::var("STY").is_ok_and(|value| !value.is_empty())
}

fn detach_screen() -> Result<()> {
    Command::new("screen").arg("-d").arg("ebay_authd").spawn()?;
    Ok(())
}
