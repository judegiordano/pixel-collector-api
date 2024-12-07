use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::AppError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            id: Self::generate_nanoid(),
            username: Default::default(),
            password: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Auth {
    pub fn generate_nanoid() -> String {
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        nanoid::nanoid!(
            20,
            &[
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ]
        )
    }

    pub fn from_attribute_map(item: HashMap<String, AttributeValue>) -> Self {
        let created_at: i64 = item
            .get("created_at")
            .unwrap()
            .as_n()
            .unwrap()
            .parse()
            .unwrap();
        let updated_at: i64 = item
            .get("updated_at")
            .unwrap()
            .as_n()
            .unwrap()
            .parse()
            .unwrap();
        Auth {
            id: item.get("id").unwrap().as_s().unwrap().to_string(),
            username: item.get("username").unwrap().as_s().unwrap().to_string(),
            password: item.get("password").unwrap().as_s().unwrap().to_string(),
            created_at: Utc.timestamp_opt(created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(updated_at, 0).unwrap(),
        }
    }

    pub async fn get_by_id(client: &Client, table_name: &str, id: &str) -> Result<Self, AppError> {
        let value = AttributeValue::S(id.to_string());
        let get = client.get_item().table_name(table_name).key("id", value);
        let output = get.send().await.map_err(AppError::not_found)?;
        let Some(item) = output.item else {
            return Err(AppError::not_found("no auth documents found"));
        };
        Ok(Self::from_attribute_map(item))
    }

    pub async fn create(&self, client: &Client, table_name: &str) -> Result<Self, AppError> {
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S(self.id.to_string()));
        item.insert(
            "username".to_string(),
            AttributeValue::S(self.username.to_string()),
        );
        item.insert(
            "password".to_string(),
            AttributeValue::S(self.password.to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::N(self.created_at.timestamp().to_string()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::N(self.updated_at.timestamp().to_string()),
        );
        client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(AppError::bad_request)?;
        Ok(self.clone())
    }
}
