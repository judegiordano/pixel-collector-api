use axum::extract::Request;
use mongoose::{doc, types::MongooseError, DateTime, IndexModel, IndexOptions, Model};
use serde::{Deserialize, Serialize};

use crate::{
    env::Env,
    errors::AppError,
    jwt::{self, Claims, Service},
    oauth::{
        self,
        google::types::{GoogleAccessToken, GoogleUserInfo},
    },
};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GoogleProviderInformation {
    pub metadata: GoogleUserInfo,
    pub tokens: GoogleAccessToken,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Auth {
    pub token_version: u32,
    pub google: GoogleProviderInformation,
    // other providers
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub auth: Auth,
    pub service: Service,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Self::generate_nanoid(),
            auth: Auth::default(),
            service: Service::LOCALHOST,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }
}

impl User {
    pub fn to_bson<T: Serialize>(data: T) -> Result<bson::Bson, AppError> {
        bson::to_bson(&data).map_err(AppError::internal_server_error)
    }

    pub async fn migrate() -> Result<Vec<String>, MongooseError> {
        let created = Self::create_indexes(&[IndexModel::builder()
            .keys(doc! { "auth.google.metadata.id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build()])
        .await?;
        Ok(created.index_names)
    }

    pub async fn create_or_update_google(
        google_user_info: GoogleUserInfo,
        token_data: GoogleAccessToken,
    ) -> Result<Self, AppError> {
        if let Ok(user) = Self::read(doc! {
            "auth.google.metadata.id": google_user_info.id.to_string()
        })
        .await
        {
            let updates = doc! {
                "auth.google": {
                    "metadata": Self::to_bson(google_user_info)?,
                    "tokens": Self::to_bson(token_data)?,
                }
            };
            return Self::update(doc! { "_id": user.id }, updates)
                .await
                .map_err(AppError::bad_request);
        };
        // else build new user
        let user = Self {
            auth: Auth {
                google: GoogleProviderInformation {
                    tokens: token_data,
                    metadata: google_user_info,
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let user = user.save().await.map_err(AppError::bad_request)?;
        Ok(user)
    }

    pub async fn refresh_google_tokens(&self, env: &Env) -> Result<Self, AppError> {
        let refresh_token = &self.auth.google.tokens.refresh_token;
        let tokens = oauth::google::refresh_tokens(
            &env.google_client_id,
            &env.google_client_secret,
            &refresh_token,
        )
        .await?;
        let updates = doc! {
            "auth.google.tokens": {
                "access_token": tokens.access_token,
                "expires_in": tokens.expires_in,
                "token_type": tokens.token_type,
                "scope": tokens.scope,
                "refresh_token": refresh_token,
            }
        };
        Self::update(doc! { "_id": &self.id }, updates)
            .await
            .map_err(AppError::internal_server_error)
    }

    pub fn sign_token(&self, secret: &str) -> Result<String, AppError> {
        jwt::sign(&self.id, self.auth.token_version, self.service, secret)
    }

    pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
        jwt::verify(token, secret)
    }

    pub async fn authenticate(req: Request, secret: &str) -> Result<Self, AppError> {
        let headers = req.headers();
        let auth = headers
            .get("authorization")
            .ok_or_else(|| AppError::unauthorized("missing auth header"))?;
        let parts = auth
            .to_str()
            .map_err(AppError::internal_server_error)?
            .split_ascii_whitespace()
            .collect::<Vec<_>>();
        let token = *parts
            .get(1)
            .ok_or_else(|| AppError::unauthorized("missing auth token"))?;
        let Claims {
            user_id,
            token_version,
            issuer,
            ..
        } = Self::verify_token(token, secret)?;
        let user = Self::read_by_id(user_id)
            .await
            .map_err(AppError::internal_server_error)?;
        if token_version != user.auth.token_version {
            return Err(AppError::unauthorized("invalid token version"));
        }
        if issuer != user.service {
            return Err(AppError::forbidden(
                "you do not have permission to access this service",
            ));
        }
        Ok(user)
    }
}

impl Model for User {}
