use aws_sdk_dynamodb::Client;
use axum::response::Response;
use moka::future::Cache;
use serde::{Deserialize, Serialize};

use crate::{
    env::{Env, Stage},
    errors::AppError,
};

pub type ApiResponse = Result<Response, AppError>;

pub const ONE_MINUTE_IN_MS: u64 = 1_000 * 60;
pub const FIVE_MINUTES_IN_MS: u64 = ONE_MINUTE_IN_MS * 5;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ping {
    pub stage: Stage,
    pub last_updated: i64,
}

#[derive(Debug, Clone)]
pub struct DynamoConnection {
    pub client: Client,
    pub table: String,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub auth_table: DynamoConnection,
    pub env: Env,
    pub stage_cache: Cache<String, Ping>,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

pub mod oauth {
    use base64::{prelude::BASE64_URL_SAFE, Engine};
    use serde::{Deserialize, Serialize};

    use crate::errors::AppError;

    #[derive(Debug, Serialize)]
    pub struct OauthLinks {
        pub google: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct State {
        pub host: u32,
        pub token: String,
    }

    impl State {
        pub fn to_base64(&self) -> Result<String, AppError> {
            let str = serde_json::to_string(&self).map_err(AppError::bad_request)?;
            Ok(BASE64_URL_SAFE.encode(str))
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct GoogleOauthCallback {
        pub code: String,
        pub state: String,
    }
}
