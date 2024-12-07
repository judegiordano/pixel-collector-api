use aws_sdk_dynamodb::Client;

use super::config;

pub async fn connect() -> Client {
    let config = if cfg!(debug_assertions) {
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .test_credentials()
            .endpoint_url("http://localhost:8000")
            .load()
            .await
    } else {
        config().await
    };
    Client::new(&config)
    // if cfg!(debug_assertions) {
    //     let local_url = "http://localhost:8000";
    //     let config = Config::builder().endpoint_url(local_url).build();
    //     return Client::from_conf(config);
    // }
    // let config = config().await;
    // Client::new(&config)
}
