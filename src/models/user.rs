use mongoose::{doc, types::MongooseError, DateTime, IndexModel, IndexOptions, Model};
use serde::{Deserialize, Serialize};

use crate::{
    errors::AppError,
    oauth::google::{GoogleAccessToken, GoogleUserInfo},
};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GoogleProviderInformation {
    pub metadata: GoogleUserInfo,
    pub tokens: GoogleAccessToken,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Auth {
    pub google: GoogleProviderInformation,
    // other providers
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub auth: Auth,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Self::generate_nanoid(),
            auth: Auth::default(),
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
        let exists = Self::read(doc! {
            "auth.google.metadata.id": google_user_info.id.to_string()
        })
        .await
        .map_or(None, Some);
        // user exists; update data
        if let Some(mut user) = exists {
            user.auth.google.tokens = token_data;
            user.auth.google.metadata = google_user_info;
            let auth_updates = Self::to_bson(user.auth)?;
            return Self::update(doc! { "_id": user.id }, doc! { "auth": auth_updates })
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
            },
            ..Default::default()
        };
        let user = user.save().await.map_err(AppError::bad_request)?;
        Ok(user)
    }
}

impl Model for User {}
