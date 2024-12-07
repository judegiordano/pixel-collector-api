use crate::types::AppState;

mod auth;
mod dev;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/dev", dev::router())
        .nest("/auth", auth::router())
}
