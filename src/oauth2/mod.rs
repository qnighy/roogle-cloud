use parking_lot::Mutex;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub use errors::{AuthError, OAuth2Error};

mod errors;

#[derive(Debug, Clone)]
pub struct OAuth2Client {
    client: reqwest::Client,
    inner: Arc<OAuth2Inner>,
}

impl OAuth2Client {
    pub fn new(config: OAuth2Config) -> Self {
        Self::with_client(config, reqwest::Client::new())
    }

    pub fn with_client(config: OAuth2Config, client: reqwest::Client) -> Self {
        Self {
            client,
            inner: Arc::new(OAuth2Inner::new(config)),
        }
    }

    pub async fn fetch_access_token(&self) -> Result<(), OAuth2Error> {
        match &self.inner.config {
            OAuth2Config::RefreshToken(config) => {
                let access_token = config.fetch_access_token(&self.client).await?;
                let mut state = self.inner.state.lock();
                state.access_token = Some(access_token);
            }
        }
        Ok(())
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        let access_token = {
            let state = self.inner.state.lock();
            state.access_token.as_ref().unwrap().access_token.clone()
        };
        self.client.post(url).bearer_auth(access_token)
    }
}

#[derive(Debug)]
struct OAuth2Inner {
    config: OAuth2Config,
    state: Mutex<OAuth2State>,
}

impl OAuth2Inner {
    fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            state: Mutex::new(OAuth2State::new()),
        }
    }
}

#[derive(Debug)]
struct OAuth2State {
    access_token: Option<AccessToken>,
}

impl OAuth2State {
    fn new() -> Self {
        Self { access_token: None }
    }
}

#[derive(Debug)]
struct AccessToken {
    access_token: String,
    expires_in: Duration,
    acquired_at: SystemTime,
}

#[derive(Debug, Clone)]
pub enum OAuth2Config {
    // AuthorizationCode,
    RefreshToken(OAuth2RefreshToken),
    // Password,
    // Jwt,
}

#[derive(Debug, Clone)]
pub struct OAuth2RefreshToken {
    pub token_credential_uri: String,
    pub authorization_uri: String,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub scope: Vec<String>,
}

impl OAuth2RefreshToken {
    async fn fetch_access_token(
        &self,
        client: &reqwest::Client,
    ) -> Result<AccessToken, OAuth2Error> {
        let body = AccessTokenRequestBody::RefreshToken(RefreshTokenRequestBody {
            refresh_token: self.refresh_token.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            scope: self.scope.join(" "),
        });
        let resp = client
            .post(&self.token_credential_uri)
            .form(&body)
            .send()
            .await?;
        eprintln!("resp = {:?}", resp);
        if !resp.status().is_success() {
            let body: AuthError = resp.json().await?;
            return Err(body.into());
        }
        let body: AccessTokenResponseBody = resp.json().await?;
        Ok(AccessToken {
            access_token: body.access_token,
            expires_in: Duration::from_secs(u64::from(body.expires_in)),
            acquired_at: SystemTime::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
enum AccessTokenRequestBody {
    RefreshToken(RefreshTokenRequestBody),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshTokenRequestBody {
    refresh_token: String,
    client_id: String,
    client_secret: String,
    scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessTokenResponseBody {
    access_token: String,
    token_type: String,
    expires_in: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id_token: Option<String>, // JWT
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}
