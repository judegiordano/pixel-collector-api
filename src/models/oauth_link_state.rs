use mongoose::{doc, types::MongooseError, DateTime, IndexModel, IndexOptions, Model};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Provider {
    GOOGLE,
}

impl Provider {
    pub fn to_string(&self) -> String {
        match self {
            Self::GOOGLE => "GOOGLE".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LinkState {
    #[serde(rename = "_id")]
    pub id: String,
    pub provider: Provider,
    pub redirect: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl LinkState {
    pub fn build_redirect(host: &str) -> String {
        match host {
            //
            _ => "http://localhost:3000/oauth/google-redirect".to_string(),
        }
    }

    pub async fn migrate() -> Result<Vec<String>, MongooseError> {
        let expiration = Duration::from_secs(60 * 10);
        let created = Self::create_indexes(&[IndexModel::builder()
            .keys(doc! { "created_at": 1 })
            .options(IndexOptions::builder().expire_after(expiration).build())
            .build()])
        .await?;
        Ok(created.index_names)
    }
}

impl Default for LinkState {
    fn default() -> Self {
        Self {
            id: Self::generate_nanoid(),
            redirect: String::default(),
            provider: Provider::GOOGLE,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }
}

impl Model for LinkState {}
