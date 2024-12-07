use lambda_http::Error;
use pixel_collector_api::{
    cache,
    controllers::routes,
    logger,
    types::{AppState, ONE_MINUTE_IN_MS},
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    logger::init()?;
    let state = AppState {
        env_cache: cache::prepare(10_000, ONE_MINUTE_IN_MS),
    };
    let app = axum::Router::new().nest("/", routes()).with_state(state);
    if cfg!(debug_assertions) {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("listening on {:?}", listener.local_addr()?);
        return Ok(axum::serve(listener, app).await?);
    }
    lambda_http::run(app).await
}
