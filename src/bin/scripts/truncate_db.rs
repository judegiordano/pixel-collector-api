use pixel_collector_api::{aws::dynamo::connect, env::Env, errors::AppError};
use std::collections::HashMap;

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let client = connect().await;
    let env = Env::load()?;
    let table = env.auth_table_name;
    let output = client
        .scan()
        .table_name(table.clone())
        .send()
        .await
        .map_err(AppError::bad_request)?;
    for item in output.items() {
        let id = item.get("id").unwrap();
        let mut key = HashMap::new();
        key.insert("id".to_string(), id.clone());
        client
            .delete_item()
            .table_name(table.clone())
            .set_key(Some(key))
            .send()
            .await
            .map_err(AppError::internal_server_error)?;
        println!("deleting {id:#?}");
    }
    Ok(())
}
