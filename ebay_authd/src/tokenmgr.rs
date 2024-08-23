use crate::error::{Error, Result};
use log::info;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::http_client,
    EmptyExtraTokenFields, RefreshToken, StandardTokenResponse, TokenResponse,
};
use std::time::{Duration, Instant};

type TokenResult = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[derive(Debug)]
pub struct TokenManager {
    client: BasicClient,
    token: TokenResult,
    refresh_token: RefreshToken,
    refresh: Instant,
}

impl TokenManager {
    #[must_use]
    pub fn new(client: BasicClient, token: TokenResult) -> Self {
        let refresh_token = token.refresh_token().cloned().unwrap();

        Self {
            client,
            token,
            refresh_token,
            refresh: Instant::now(),
        }
    }

    #[must_use]
    pub fn get_token(&self) -> String {
        self.token.access_token().secret().to_string()
    }

    #[must_use]
    pub fn get_token_bytes(&self) -> Box<[u8]> {
        self.token.access_token().secret().bytes().collect()
    }

    #[must_use]
    pub fn expires_soon(&self) -> bool {
        self.expiry() <= Duration::from_secs(10)
    }

    #[must_use]
    pub fn expiry(&self) -> Duration {
        self.token.expires_in().unwrap() - self.refresh.elapsed()
    }

    pub fn refresh(&mut self) -> Result<()> {
        info!("Refreshing token");
        let new_token = self
            .client
            .exchange_refresh_token(&self.refresh_token)
            .request(http_client)
            .map_err(|_| Error::TokenRequest)?;

        self.refresh = Instant::now();
        self.token = new_token;

        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        if !self.expires_soon() {
            return Ok(());
        }

        self.refresh()?;

        Ok(())
    }
}
