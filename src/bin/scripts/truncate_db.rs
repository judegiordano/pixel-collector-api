use dotenv::dotenv;
use pixel_collector_api::{
    aws::dynamo::{connect, Table},
    errors::AppError,
    models::auth::Auth,
};
use std::collections::HashMap;

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    let client = connect().await;
    let output = client
        .scan()
        .table_name(Auth::table_name())
        .send()
        .await
        .map_err(AppError::bad_request)?;
    for item in output.items() {
        let id = item.get("id").unwrap();
        let mut key = HashMap::new();
        key.insert("id".to_string(), id.clone());
        client
            .delete_item()
            .table_name(Auth::table_name())
            .set_key(Some(key))
            .send()
            .await
            .map_err(AppError::internal_server_error)?;
        println!("deleting {id:#?}");
    }
    Ok(())
}
