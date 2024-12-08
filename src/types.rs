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
