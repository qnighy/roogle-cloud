use roogle_cloud::auth::DefaultCredential;
use roogle_cloud::oauth2::{OAuth2Client, OAuth2Config, OAuth2RefreshToken};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub const TOKEN_CREDENTIAL_URI: &str = "https://oauth2.googleapis.com/token";
pub const AUDIENCE: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct QueryRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    // default_dataset: {
    //   object (DatasetReference)
    // },
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dry_run: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preserve_nulls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    use_query_cache: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    use_legacy_sql: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parameter_mode: Option<String>,
    // queryParameters: [
    //   {
    //     object (QueryParameter)
    //   }
    // ],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home = PathBuf::from(env::var_os("HOME").expect("HOME not found"));
    let file = File::open(home.join(".config/gcloud/application_default_credentials.json"))?;
    let file = BufReader::new(file);
    let credential: DefaultCredential = serde_json::from_reader(file)?;
    eprintln!("credential = {:?}", credential);

    let credential = match &credential {
        DefaultCredential::AuthorizedUser(cred) => cred,
        DefaultCredential::__NonExhaustive => unreachable!(),
        _ => panic!("invalid credential type"),
    };

    let config = OAuth2RefreshToken {
        token_credential_uri: String::from("https://oauth2.googleapis.com/token"),
        authorization_uri: String::from("https://accounts.google.com/o/oauth2/auth"),
        client_id: credential.client_id.clone(),
        client_secret: credential.client_secret.clone(),
        refresh_token: credential.refresh_token.clone(),
        // scope: vec![String::from("https://www.googleapis.com/auth/bigquery")],
        scope: vec![],
    };
    let config = OAuth2Config::RefreshToken(config);

    let client = OAuth2Client::new(config);
    client.fetch_access_token().await?;

    let project_id =
        env::var("GOOGLE_CLOUD_PROJECT").expect("GOOGLE_CLOUD_PROJECT not found or invalid");

    let url = format!(
        "https://bigquery.googleapis.com/bigquery/v2/projects/{project_id}/queries",
        project_id = project_id
    );
    let req = QueryRequest {
        query: String::from("SELECT 1"),
        ..Default::default()
    };
    let resp = client.post(&url).json(&req).send().await?;
    eprintln!("resp = {:?}", resp);
    let resp_body: serde_json::Value = resp.json().await?;
    eprintln!("resp_body = {:?}", resp_body);

    Ok(())
}
