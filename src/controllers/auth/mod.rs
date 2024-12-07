use axum::routing::{get, post};

use crate::types::AppState;

mod controller;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/:id", get(controller::read_by_id))
        .route("/", post(controller::create))
}
