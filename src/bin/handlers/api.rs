use lambda_http::Error;
use pixel_collector_api::{
    aws::dynamo::{self},
    cache,
    controllers::routes,
    env::Env,
    logger,
    types::{AppState, DynamoConnection, ONE_MINUTE_IN_MS},
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    logger::init()?;
    let env = Env::load()?;
    let connection = dynamo::connect().await;
    let state = AppState {
        auth_table: DynamoConnection {
            client: connection,
            table: env.auth_table_name.to_string(),
        },
        env,
        stage_cache: cache::prepare(10_000, ONE_MINUTE_IN_MS),
    };
    let app = axum::Router::new().nest("/", routes()).with_state(state);
    if cfg!(debug_assertions) {
        let listener = tokio::net::TcpListener::bind("localhost:3000").await?;
        tracing::info!("listening on {:?}", listener.local_addr()?);
        return Ok(axum::serve(listener, app).await?);
    }
    lambda_http::run(app).await
}
