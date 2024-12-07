use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use crate::{
    models::auth::Auth,
    types::{ApiResponse, AppState},
};

pub async fn read_by_id(State(state): State<AppState>, Path(id): Path<String>) -> ApiResponse {
    let item = Auth::get_by_id(
        &state.auth_table_client,
        &state.env.auth_table_name,
        &id.to_string(),
    )
    .await?;
    Ok(Json(item).into_response())
}

pub async fn create(State(state): State<AppState>) -> ApiResponse {
    let new = Auth {
        username: format!("username_{}", Auth::generate_nanoid()),
        password: format!("password_{}", Auth::generate_nanoid()),
        ..Default::default()
    };
    let inserted = new
        .create(&state.auth_table_client, &state.env.auth_table_name)
        .await?;
    Ok(Json(inserted).into_response())
}
