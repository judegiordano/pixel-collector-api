use aws_sdk_s3::{
    operation::{
        delete_object::DeleteObjectOutput, get_object::GetObjectOutput, put_object::PutObjectOutput,
    },
    presigning::PresigningConfig,
    Client,
};
use std::time::Duration;

use crate::errors::AppError;

use super::config;

pub struct Bucket {
    pub name: String,
    pub client: Client,
}

impl Bucket {
    pub async fn new(bucket_name: impl ToString) -> Self {
        Self {
            name: bucket_name.to_string(),
            client: Client::new(&config().await),
        }
    }

    fn _build_presigned_config(duration: Duration) -> Result<PresigningConfig, AppError> {
        PresigningConfig::expires_in(duration).map_err(AppError::bad_request)
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub async fn get_object(&self, key: impl ToString) -> Result<GetObjectOutput, AppError> {
        self.client
            .get_object()
            .bucket(&self.name)
            .key(key.to_string())
            .send()
            .await
            .map_err(AppError::internal_server_error)
    }

    pub async fn put_object(
        &self,
        key: impl ToString,
        body: impl IntoIterator<Item = u8>,
    ) -> Result<PutObjectOutput, AppError> {
        let stream = body.into_iter().collect::<Vec<_>>();
        self.client
            .put_object()
            .bucket(&self.name)
            .body(stream.into())
            .key(key.to_string())
            .send()
            .await
            .map_err(AppError::internal_server_error)
    }

    pub async fn delete_object(&self, key: impl ToString) -> Result<DeleteObjectOutput, AppError> {
        self.client
            .delete_object()
            .bucket(&self.name)
            .key(key.to_string())
            .send()
            .await
            .map_err(AppError::internal_server_error)
    }

    pub async fn get_presigned_url(
        &self,
        key: impl ToString,
        expires_in: Duration,
    ) -> Result<String, AppError> {
        let request = self
            .client
            .get_object()
            .bucket(&self.name)
            .key(key.to_string())
            .presigned(Self::_build_presigned_config(expires_in)?)
            .await
            .map_err(AppError::internal_server_error)?;
        Ok(request.uri().to_string())
    }

    pub async fn put_presigned_url(
        &self,
        key: impl ToString,
        expires_in: Duration,
    ) -> Result<String, AppError> {
        let request = self
            .client
            .put_object()
            .bucket(&self.name)
            .key(key.to_string())
            .presigned(Self::_build_presigned_config(expires_in)?)
            .await
            .map_err(AppError::internal_server_error)?;
        Ok(request.uri().to_string())
    }
}
