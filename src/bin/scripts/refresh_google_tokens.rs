use bson::doc;
use mongoose::{types::ListOptions, Model};
use pixel_collector_api::{env::Env, errors::AppError, logger, models::user::User};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    logger::init()?;
    let env = Env::load()?;
    let users = User::list(
        doc! {},
        ListOptions {
            limit: 0,
            ..Default::default()
        },
    )
    .await
    .map_err(AppError::not_found)?;
    for user in users {
        let updated = user.refresh_google_tokens(&env).await?;
        tracing::info!("user updated: {:?}", updated.id);
    }
    Ok(())
}
