use crate::types::AppState;

mod dev;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().nest("/dev", dev::router())
}
