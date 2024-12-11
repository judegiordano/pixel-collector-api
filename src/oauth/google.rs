use mongoose::{doc, Model};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::{
    errors::AppError,
    models::oauth_link_state::{LinkState, Provider},
    types::oauth::GoogleOauthCallback,
};

const GOOGLE_OAUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = " https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_ENDPOINT: &str = " https://www.googleapis.com/oauth2/v1/userinfo";
const GOOGLE_SCOPES: [&str; 3] = [
    "openid",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GoogleAccessToken {
    pub access_token: String,
    pub expires_in: u32, // seconds
    pub token_type: String,
    pub scope: String,
    pub refresh_token: String,
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

pub async fn build_oauth_link(client_id: &str, host: &str) -> Result<String, AppError> {
    let link_state = LinkState {
        redirect: LinkState::build_redirect(host),
        provider: Provider::GOOGLE,
        ..Default::default()
    }
    .save()
    .await
    .map_err(AppError::bad_request)?;
    let query = [
        ("client_id", client_id.to_string()),
        ("access_type", "offline".to_string()),
        ("redirect_uri", link_state.redirect.to_string()),
        ("response_type", "code".to_string()),
        ("prompt", "consent".to_string()),
        ("state", link_state.id),
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
    client_id: String,
    client_secret: String,
    params: GoogleOauthCallback,
) -> Result<GoogleAccessToken, AppError> {
    // assert link state was created by this service
    let link = LinkState::read_by_id(&params.state)
        .await
        .map_err(AppError::unauthorized)?;
    let query = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", link.redirect),
        ("grant_type", "authorization_code".to_string()),
        ("code", params.code),
    ];
    let response = Client::new()
        .post(GOOGLE_TOKEN_ENDPOINT)
        .form(&query)
        .send()
        .await
        .map_err(AppError::unauthorized)?;
    LinkState::delete(doc! { "_id": params.state })
        .await
        .map_err(AppError::internal_server_error)?;
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
