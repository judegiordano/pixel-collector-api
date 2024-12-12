use reqwest::{Client, Url};
use types::{GoogleAccessToken, GoogleRefreshToken, GoogleUserInfo};

use crate::{errors::AppError, models::oauth_link_state::LinkState};

pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct GoogleAccessToken {
        pub access_token: String,
        pub expires_in: u32, // seconds
        pub token_type: String,
        pub scope: String,
        pub refresh_token: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct GoogleRefreshToken {
        pub access_token: String,
        pub expires_in: u32, // seconds
        pub token_type: String,
        pub scope: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct GoogleUserInfo {
        pub id: String,
        pub email: String,
        pub verified_email: bool,
        pub name: String,
        pub given_name: String,
        pub family_name: String,
        pub picture: String,
    }
}

const GOOGLE_OAUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = " https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_ENDPOINT: &str = " https://www.googleapis.com/oauth2/v1/userinfo";
const GOOGLE_SCOPES: [&str; 3] = [
    "openid",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

pub async fn build_oauth_link(client_id: &str, state: &LinkState) -> Result<String, AppError> {
    let query = [
        ("client_id", client_id.to_string()),
        ("access_type", "offline".to_string()),
        ("redirect_uri", state.redirect.to_string()),
        ("response_type", "code".to_string()),
        ("prompt", "consent".to_string()),
        ("state", state.id.to_string()),
        ("scope", GOOGLE_SCOPES.join(" ")),
        ("include_granted_scopes", "true".to_string()),
    ];
    let mut url = Url::parse(GOOGLE_OAUTH_ENDPOINT).map_err(AppError::bad_request)?;
    let query_string = query
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    url.set_query(Some(&query_string));
    Ok(url.to_string())
}

pub async fn handle_callback(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect: &str,
) -> Result<GoogleAccessToken, AppError> {
    let query = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect),
        ("grant_type", "authorization_code"),
        ("code", code),
    ];
    let response = Client::new()
        .post(GOOGLE_TOKEN_ENDPOINT)
        .form(&query)
        .send()
        .await
        .map_err(AppError::unauthorized)?;
    response.json().await.map_err(AppError::unauthorized)
}

pub async fn fetch_user_info(access_token: &str) -> Result<GoogleUserInfo, AppError> {
    let query = [("alt", "json"), ("access_token", access_token)];
    let response = Client::new()
        .get(GOOGLE_USER_INFO_ENDPOINT)
        .query(&query)
        .send()
        .await
        .map_err(AppError::not_found)?;
    response.json().await.map_err(AppError::unauthorized)
}

pub async fn refresh_tokens(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<GoogleRefreshToken, AppError> {
    let query = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    let response = Client::new()
        .post(GOOGLE_TOKEN_ENDPOINT)
        .form(&query)
        .send()
        .await
        .map_err(AppError::not_found)?;
    response.json().await.map_err(AppError::unauthorized)
}
