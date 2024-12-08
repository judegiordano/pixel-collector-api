use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{aws::dynamo::DynamoHelper, errors::AppError, types::DynamoConnection};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub id: String,
    pub username: String,
    pub password: String,
    pub metadata: Option<Value>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl DynamoHelper for Auth {}

impl Default for Auth {
    fn default() -> Self {
        Self {
            id: Self::generate_nanoid(),
            username: Default::default(),
            password: Default::default(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

pub const USERNAME_IDX: &str = "username_idx";

impl Auth {
    pub async fn get_by_id(conn: &DynamoConnection, id: &str) -> Result<Self, AppError> {
        let value = AttributeValue::S(id.to_string());
        let get = conn
            .client
            .get_item()
            .table_name(&conn.table)
            .key("id", value);
        let output = get.send().await.map_err(AppError::not_found)?;
        let Some(item) = output.item else {
            return Err(AppError::not_found("no auth documents found"));
        };
        Self::from_attribute_map(&item)
    }

    pub async fn register(&self, conn: &DynamoConnection) -> Result<Self, AppError> {
        let item = self.to_attribute_map()?;
        conn.client
            .put_item()
            .table_name(&conn.table)
            .set_item(Some(item))
            .send()
            .await
            .map_err(AppError::bad_request)?;
        Ok(self.clone())
    }

    pub async fn login(conn: &DynamoConnection, username: &str) -> Result<Self, AppError> {
        let get = conn
            .client
            .query()
            .table_name(&conn.table)
            .index_name(USERNAME_IDX)
            .key_condition_expression("#username = :username")
            .expression_attribute_names("#username", "username")
            .expression_attribute_values(":username", AttributeValue::S(username.to_string()));
        let output = get.send().await.map_err(AppError::bad_request)?;
        let items = match output.items {
            Some(items) => items,
            None => return Err(AppError::not_found("no auth documents found")),
        };
        items.first().map_or_else(
            || Err(AppError::not_found("no auth documents found")),
            |exists| Self::from_attribute_map(exists),
        )
    }
}
