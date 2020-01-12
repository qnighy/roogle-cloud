use serde::{Deserialize, Serialize};
use std::env::{self, VarError};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DefaultCredential {
    ServiceAccount(ServiceAccountCredential),
    AuthorizedUser(AuthorizedUserCredential),
    #[serde(skip)]
    #[doc(hidden)]
    __NonExhaustive,
}

impl DefaultCredential {
    pub fn from_env() -> Result<Self, FromEnvError> {
        let account_type = var("GOOGLE_ACCOUNT_TYPE")?;
        Ok(if account_type == "service_account" {
            DefaultCredential::ServiceAccount(ServiceAccountCredential::from_env()?)
        } else if account_type == "authorized_user" {
            DefaultCredential::AuthorizedUser(AuthorizedUserCredential::from_env()?)
        } else {
            return Err(FromEnvError::InvalidEnv {
                name: "GOOGLE_ACCOUNT_TYPE".to_owned(),
            });
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ServiceAccountCredential {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key_id: Option<String>,
    pub private_key: String,
    pub client_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_provider_x509_cert_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_x509_cert_url: Option<String>,
    #[serde(skip)]
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl ServiceAccountCredential {
    pub fn from_env() -> Result<Self, FromEnvError> {
        let private_key = simple_unescape(var("GOOGLE_PRIVATE_KEY")?);
        let client_email = var("GOOGLE_CLIENT_EMAIL")?;
        let project_id = var_opt("GOOGLE_PROJECT_ID")?;
        Ok(Self {
            private_key,
            client_email,
            project_id,
            ..Self::default()
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthorizedUserCredential {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip)]
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl AuthorizedUserCredential {
    pub fn from_env() -> Result<Self, FromEnvError> {
        let client_id = var("GOOGLE_CLIENT_ID")?;
        let client_secret = var("GOOGLE_CLIENT_SECRET")?;
        let refresh_token = var("GOOGLE_REFRESH_TOKEN")?;
        let project_id = var_opt("GOOGLE_PROJECT_ID")?;
        Ok(Self {
            client_id,
            client_secret,
            refresh_token,
            project_id,
            __non_exhaustive: (),
        })
    }
}

#[derive(Debug, Error)]
pub enum FromEnvError {
    #[error("Environment variable not found: {name}")]
    EnvNotPresent { name: String },
    #[error("Invalid environment variable: {name}")]
    InvalidEnv { name: String },
}

fn var(k: &str) -> Result<String, FromEnvError> {
    match env::var(k) {
        Ok(x) => Ok(x),
        Err(VarError::NotPresent) => Err(FromEnvError::EnvNotPresent { name: k.to_owned() }),
        Err(VarError::NotUnicode(_)) => Err(FromEnvError::InvalidEnv { name: k.to_owned() }),
    }
}

fn var_opt(k: &str) -> Result<Option<String>, FromEnvError> {
    match env::var(k) {
        Ok(x) => Ok(Some(x)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(FromEnvError::InvalidEnv { name: k.to_owned() }),
    }
}

fn simple_unescape(s: String) -> String {
    let mut s: Vec<u8> = s.into();
    let mut i = 0;
    let mut j = 0;
    while j < s.len() {
        if j + 1 < s.len() && s[j] == b'\\' && s[j + 1] == b'n' {
            s[i] = b'\n';
            i += 1;
            j += 2;
        } else {
            i += 1;
            j += 1;
        }
    }
    s.resize(i, b'\0');
    if s.len() >= 2 && s[0] == b'"' && s[s.len() - 1] == b'"' {
        s.remove(0);
        s.resize(s.len() - 1, b'\0');
    }
    String::from_utf8(s).unwrap()
}
