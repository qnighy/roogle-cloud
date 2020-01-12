use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuth2Error {
    #[error("HTTP request error")]
    HttpError(#[from] reqwest::Error),
    #[error("")]
    AuthError(#[from] AuthError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError {
    pub error: AuthErrorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error.as_str())?;
        if let Some(description) = &self.error_description {
            write!(f, ": {}", description)?;
        }
        if let Some(uri) = &self.error_uri {
            write!(f, " ({})", uri)?;
        }
        Ok(())
    }
}

impl StdError for AuthError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum AuthErrorKind {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    UnsupportedGrantType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

impl AuthErrorKind {
    pub fn as_str(&self) -> &str {
        use AuthErrorKind::*;

        match self {
            InvalidRequest => "invalid_request",
            InvalidClient => "invalid_client",
            InvalidGrant => "invalid_grant",
            UnauthorizedClient => "unauthorized_client",
            AccessDenied => "access_denied",
            UnsupportedResponseType => "unsupported_response_type",
            UnsupportedGrantType => "unsupported_grant_type",
            InvalidScope => "invalid_scope",
            ServerError => "server_error",
            TemporarilyUnavailable => "temporarily_unavaiable",
        }
    }
}

impl TryFrom<String> for AuthErrorKind {
    type Error = String;
    fn try_from(x: String) -> Result<Self, Self::Error> {
        use AuthErrorKind::*;

        Ok(match x.as_str() {
            "invalid_request" => InvalidRequest,
            "invalid_client" => InvalidClient,
            "invalid_grant" => InvalidGrant,
            "unauthorized_client" => UnauthorizedClient,
            "access_denied" => AccessDenied,
            "unsupported_response_type" => UnsupportedResponseType,
            "unsupported_grant_type" => UnsupportedGrantType,
            "invalid_scope" => InvalidScope,
            "server_error" => ServerError,
            "temporarily_unavaiable" => TemporarilyUnavailable,
            _ => return Err(x),
        })
    }
}

impl From<AuthErrorKind> for String {
    fn from(x: AuthErrorKind) -> Self {
        x.as_str().to_owned()
    }
}
