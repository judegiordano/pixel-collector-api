use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    aws::dynamo::Table,
    models::auth::Auth,
    types::{ApiResponse, AppState},
};

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
}

pub async fn read_by_id(State(state): State<AppState>, Path(id): Path<String>) -> ApiResponse {
    let item = Auth::get_by_id(&state.auth_table, &id).await?;
    Ok(Json(item).into_response())
}

pub async fn login(State(state): State<AppState>, Json(body): Json<Login>) -> ApiResponse {
    let item = Auth::login(&state.auth_table, &body.username).await?;
    Ok(Json(item).into_response())
}

pub async fn register(State(state): State<AppState>) -> ApiResponse {
    let new = Auth {
        username: format!("username_{}", Auth::generate_nanoid()),
        password: format!("password_{}", Auth::generate_nanoid()),
        metadata: Some(json!({
            "age": 27,
            "hobbies": ["coding", "video games"],
            "address": {
                "street": 123,
                "name": "fake street"
            }
        })),
        ..Default::default()
    };
    let inserted = new.register(&state.auth_table).await?;
    Ok(Json(inserted).into_response())
}
