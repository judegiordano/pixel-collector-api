use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;

use crate::{
    env::Env,
    types::{ApiResponse, AppState, Ping},
};

pub async fn ping(State(state): State<AppState>) -> ApiResponse {
    if let Some(stage) = state.env_cache.get("stage").await {
        return Ok(Json(stage).into_response());
    }
    let Env { stage, .. } = Env::load()?;
    let ping = Ping {
        stage,
        last_updated: Utc::now().timestamp_millis(),
    };
    state
        .env_cache
        .insert("stage".to_string(), ping.clone())
        .await;
    Ok(Json(ping).into_response())
}
