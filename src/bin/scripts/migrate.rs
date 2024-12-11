use pixel_collector_api::{
    errors::AppError,
    logger,
    models::{oauth_link_state::LinkState, user::User},
};
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    logger::init()?;
    let indexes = try_join!(LinkState::migrate(), User::migrate())
        .map_err(AppError::internal_server_error)?;
    tracing::info!("{:#?}", indexes);
    Ok(())
}
