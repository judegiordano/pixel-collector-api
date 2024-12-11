use crate::{
    models::user::User,
    oauth::{self},
    types::{
        oauth::{GoogleOauthCallback, OauthLinks},
        ApiResponse, AppState,
    },
};
use axum::{
    extract::{Host, Query, State},
    response::IntoResponse,
    Json,
};

pub async fn get_oauth_links(State(state): State<AppState>, Host(host): Host) -> ApiResponse {
    let google = oauth::google::build_oauth_link(&state.env.google_client_id, &host).await?;
    let links = OauthLinks { google };
    Ok(Json(links).into_response())
}

pub async fn google_redirect_handler(
    State(state): State<AppState>,
    Query(query): Query<GoogleOauthCallback>,
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

fn _refresh() {
    // POST /token HTTP/1.1
    // Host: oauth2.googleapis.com
    // Content-Type: application/x-www-form-urlencoded

    // client_id=your_client_id&
    // client_secret=your_client_secret&
    // refresh_token=refresh_token&
    // grant_type=refresh_token

    // {
    //     "access_token": "1/fFAGRNJru1FTz70BzhT3Zg",
    //     "expires_in": 3920,
    //     "scope": "https://www.googleapis.com/auth/drive.metadata.readonly https://www.googleapis.com/auth/calendar.readonly",
    //     "token_type": "Bearer"
    // }
}
