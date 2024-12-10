use axum::routing::get;

use crate::types::AppState;

mod controller;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().route("/google", get(controller::ping))
}
