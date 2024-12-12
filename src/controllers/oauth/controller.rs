use crate::{
    errors::AppError,
    jwt::Service,
    models::{
        oauth_link_state::{LinkState, Provider},
        user::User,
    },
    oauth::{self},
    types::{ApiResponse, AppState},
};
use axum::{
    extract::{Query, Request, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use mongoose::Model;
use serde_json::json;

pub async fn get_oauth_links(State(state): State<AppState>, headers: HeaderMap) -> ApiResponse {
    let link_state = LinkState {
        service: Service::from_headers(headers)?,
        provider: Provider::GOOGLE,
        ..Default::default()
    }
    .save()
    .await
    .map_err(AppError::bad_request)?;
    let google = oauth::google::build_oauth_link(&state.env.google_client_id, &link_state).await?;
    let links = oauth::types::Links { google };
    Ok(Json(links).into_response())
}

pub async fn user(State(state): State<AppState>, req: Request) -> ApiResponse {
    let user = User::authenticate(req, &state.env.jwt_secret).await?;
    Ok(Json(user).into_response())
}

pub async fn google_redirect_handler(
    State(state): State<AppState>,
    Query(query): Query<oauth::types::GoogleOauthCallback>,
) -> ApiResponse {
    // assert link state was created by this service
    let link = LinkState::read_by_id(&query.state)
        .await
        .map_err(AppError::unauthorized)?;
    let token_data = oauth::google::handle_callback(
        &state.env.google_client_id,
        &state.env.google_client_secret,
        &query.code,
        &link.redirect,
    )
    .await?;
    let user_metadata = oauth::google::fetch_user_info(&token_data.access_token).await?;
    let user = User::create_or_update_google(link.service, user_metadata, token_data).await?;
    let token = user.sign_token(&state.env.jwt_secret)?;
    Ok(Json(json!({ "token": token })).into_response())
}
