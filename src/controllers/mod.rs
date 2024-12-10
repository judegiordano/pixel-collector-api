use crate::types::AppState;

mod auth;
mod dev;
mod oauth;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .nest("/dev", dev::router())
        .nest("/auth", auth::router())
        .nest("/oauth", auth::router())
}
