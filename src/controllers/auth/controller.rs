use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

use crate::{
    models::auth::Auth,
    types::{ApiResponse, AppState, Login},
};

pub async fn read_by_id(State(state): State<AppState>, Path(id): Path<String>) -> ApiResponse {
    let item = Auth::get_by_id(&state.dynamo, &id).await?;
    Ok(Json(item).into_response())
}

pub async fn login(State(state): State<AppState>, Json(body): Json<Login>) -> ApiResponse {
    let item = Auth::login(&state.dynamo, &body.username).await?;
    Ok(Json(item).into_response())
}

pub async fn register(State(state): State<AppState>, Json(body): Json<Login>) -> ApiResponse {
    let mut new = Auth {
        username: body.username,
        password: body.password,
        metadata: None,
        ..Default::default()
    };
    let inserted = new.register(&state.dynamo).await?;
    Ok(Json(inserted).into_response())
}
