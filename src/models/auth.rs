use aws_sdk_dynamodb::{operation::query::QueryOutput, types::AttributeValue};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::{aws::dynamo::Table, errors::AppError, types::DynamoConnection};

pub const USERNAME_IDX: &str = "username_idx";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub id: String,
    pub username: String,
    pub password: String,
    pub metadata: Option<Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Table for Auth {}

impl Default for Auth {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_nanoid(),
            username: String::default(),
            password: String::default(),
            metadata: None,
            created_at: now.timestamp_millis(),
            updated_at: now.timestamp_millis(),
        }
    }
}

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

    async fn get_by_username_query(
        conn: &DynamoConnection,
        username: &str,
    ) -> Result<QueryOutput, AppError> {
        conn.client
            .query()
            .table_name(&conn.table)
            .index_name(USERNAME_IDX)
            .key_condition_expression("#username = :username")
            .expression_attribute_names("#username", "username")
            .expression_attribute_values(":username", AttributeValue::S(username.to_string()))
            .send()
            .await
            .map_err(AppError::bad_request)
    }

    pub async fn register(&mut self, conn: &DynamoConnection) -> Result<Self, AppError> {
        let output = Self::get_by_username_query(conn, &self.username).await?;
        if !output.items().is_empty() {
            return Err(AppError::bad_request("username taken"));
        }
        // hash password
        self.password = "HASHED".to_string();
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
        let output = Self::get_by_username_query(conn, username).await?;
        let Some(items) = output.items else {
            return Err(AppError::not_found("username not found"));
        };
        if let Some(first) = items.first() {
            return Ok(Self {
                id: first.s("id")?,
                username: first.s("username")?,
                password: first.s("password")?,
                metadata: None,
                created_at: first.n("created_at")?,
                updated_at: first.n("updated_at")?,
            });
        }
        Err(AppError::not_found("username not found"))
        // // TODO: compare password hash
        // items.first().map_or_else(
        //     || Err(AppError::not_found("username not found")),
        //     Self::from_attribute_map,
        // )
    }
}

#[allow(dead_code)]
trait DynamoHelper {
    fn s(&self, key: &str) -> Result<String, AppError>;
    fn s_option(&self, key: &str) -> Result<Option<String>, AppError>;
    fn n<T: std::str::FromStr>(&self, key: &str) -> Result<T, AppError>;
    fn n_option<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, AppError>;
}

impl DynamoHelper for &HashMap<String, AttributeValue> {
    fn s(&self, key: &str) -> Result<String, AppError> {
        let value = match self.get(key) {
            Some(exists) => exists,
            None => {
                tracing::error!("[ERROR]: cannot find {key} in {self:?}");
                return Err(AppError::internal_server_error("error parsing key"));
            }
        };
        match value.as_s() {
            Ok(value) => Ok(value.to_string()),
            Err(err) => {
                tracing::error!("[ERROR]: cannot parse {key} {err:?}");
                Err(AppError::internal_server_error("error parsing value"))
            }
        }
    }
    fn s_option(&self, key: &str) -> Result<Option<String>, AppError> {
        let value = match self.get(key) {
            Some(exists) => exists,
            None => return Ok(None),
        };
        match value.as_s() {
            Ok(value) => Ok(Some(value.to_string())),
            Err(err) => {
                tracing::error!("[ERROR]: cannot parse {key} {err:?}");
                Err(AppError::internal_server_error("error parsing value"))
            }
        }
    }

    fn n<T: std::str::FromStr>(&self, key: &str) -> Result<T, AppError> {
        let value = match self.get(key) {
            Some(exists) => exists,
            None => {
                tracing::error!("[ERROR]: cannot find {key} in {self:?}");
                return Err(AppError::internal_server_error("error parsing key"));
            }
        };
        let num = match value.as_n() {
            Ok(value) => value.to_string(),
            Err(err) => {
                tracing::error!("[ERROR]: cannot parse {key} {err:?}");
                return Err(AppError::internal_server_error("error parsing value"));
            }
        };
        match num.parse::<T>() {
            Ok(num) => Ok(num),
            Err(_) => {
                tracing::error!("[ERROR]: cannot parse number {key}");
                return Err(AppError::internal_server_error(
                    "error parsing value as number",
                ));
            }
        }
    }
    fn n_option<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, AppError> {
        let value = match self.get(key) {
            Some(exists) => exists,
            None => return Ok(None),
        };
        let num = match value.as_n() {
            Ok(value) => value.to_string(),
            Err(err) => {
                tracing::error!("[ERROR]: cannot parse {key} {err:?}");
                return Err(AppError::internal_server_error("error parsing value"));
            }
        };
        match num.parse::<T>() {
            Ok(num) => Ok(Some(num)),
            Err(_) => {
                tracing::error!("[ERROR]: cannot parse number {key}");
                return Err(AppError::internal_server_error(
                    "error parsing value as number",
                ));
            }
        }
    }
}
