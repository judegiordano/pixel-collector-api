use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, fmt::Debug};

use crate::errors::AppError;

use super::config;

pub async fn connect() -> Client {
    let config = if cfg!(debug_assertions) {
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .test_credentials()
            .endpoint_url("http://localhost:8000")
            .load()
            .await
    } else {
        config().await
    };
    Client::new(&config)
}

pub trait Table: Serialize + DeserializeOwned + Debug {
    fn table_name() -> &'static str {
        "pixel_collector_users"
    }

    fn _safe_parse(string: &str) -> u64 {
        match string.parse() {
            Ok(parsed) => parsed,
            Err(err) => {
                tracing::error!("[ERROR]: error parsing integer: {err:?}");
                0
            }
        }
    }

    fn generate_nanoid() -> String {
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

    fn _attribute_to_serde_value(attribute: &AttributeValue) -> Value {
        match attribute {
            AttributeValue::Bool(bool) => Value::Bool(*bool),
            AttributeValue::L(vec) => Value::Array(
                vec.iter()
                    .map(|v| Self::_attribute_to_serde_value(v))
                    .collect::<Vec<_>>(),
            ),
            AttributeValue::M(hash_map) => match Self::from_attribute_map(hash_map) {
                Ok(map) => map,
                Err(err) => {
                    tracing::error!("[ERROR]: error parsing struct: {err:?}");
                    Value::Null
                }
            },
            AttributeValue::N(number) => Value::Number(Self::_safe_parse(number).into()),
            AttributeValue::Ns(vec) => Value::Array(
                vec.iter()
                    .map(|v| Value::Number(Self::_safe_parse(v).into()))
                    .collect::<Vec<_>>(),
            ),
            AttributeValue::Null(_) => Value::Null,
            AttributeValue::S(str) => Value::String(str.to_string()),
            AttributeValue::Ss(vec) => Value::Array(
                vec.iter()
                    .map(|v| Value::String(v.to_string()))
                    .collect::<Vec<_>>(),
            ),
            catch_all => {
                tracing::warn!("{catch_all:?} conversion not supported");
                Value::Null
            }
        }
    }

    fn _serde_value_to_attribute(value: &Value) -> AttributeValue {
        match value {
            Value::Null => AttributeValue::Null(true),
            Value::Bool(val) => AttributeValue::Bool(*val),
            Value::Number(number) => AttributeValue::N(number.to_string()),
            Value::String(str) => AttributeValue::S(str.clone()),
            Value::Array(vec) => AttributeValue::L(
                vec.iter()
                    .map(|v| Self::_serde_value_to_attribute(v))
                    .collect(),
            ),
            Value::Object(map) => {
                let object = map.iter().fold(HashMap::new(), |mut acc, (key, value)| {
                    acc.insert(key.to_string(), Self::_serde_value_to_attribute(value));
                    acc
                });
                AttributeValue::M(object)
            }
        }
    }

    fn from_attribute_map<T: DeserializeOwned>(
        map: &HashMap<String, AttributeValue>,
    ) -> Result<T, AppError> {
        let json = map.iter().fold(Map::default(), |mut acc, (key, value)| {
            acc.insert(key.to_string(), Self::_attribute_to_serde_value(value));
            acc
        });
        let str = serde_json::to_string(&json).map_err(AppError::internal_server_error)?;
        serde_json::from_str(&str).map_err(AppError::internal_server_error)
    }

    fn to_attribute_map(&self) -> Result<HashMap<String, AttributeValue>, AppError> {
        let json = serde_json::to_value(self).map_err(AppError::bad_request)?;
        let Some(object) = json.as_object() else {
            tracing::error!("[ERROR]: converting {self:?}");
            return Err(AppError::internal_server_error(
                "error converting to object",
            ));
        };
        let map = object.iter().fold(HashMap::new(), |mut acc, (key, value)| {
            acc.insert(key.to_string(), Self::_serde_value_to_attribute(value));
            acc
        });
        Ok(map)
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
