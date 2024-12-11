use axum::routing::get;

use crate::types::AppState;

mod controller;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(controller::get_oauth_links))
        // google
        .route("/google-redirect", get(controller::google_redirect_handler))
}
