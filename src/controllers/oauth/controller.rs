use crate::{
    models::user::User,
    oauth::{self},
    types::{ApiResponse, AppState},
};
use axum::{
    extract::{Host, Query, State},
    response::IntoResponse,
    Json,
};

pub async fn get_oauth_links(State(state): State<AppState>, Host(host): Host) -> ApiResponse {
    let google = oauth::google::build_oauth_link(&state.env.google_client_id, &host).await?;
    let links = oauth::types::Links { google };
    Ok(Json(links).into_response())
}

pub async fn google_redirect_handler(
    State(state): State<AppState>,
    Query(query): Query<oauth::types::GoogleOauthCallback>,
) -> ApiResponse {
    let token_data = oauth::google::handle_callback(
        state.env.google_client_id,
        state.env.google_client_secret,
        query,
    )
    .await?;
    let user_metadata = oauth::google::fetch_user_info(&token_data.access_token).await?;
    let user = User::create_or_update_google(user_metadata, token_data).await?;
    Ok(Json(user).into_response())
}
