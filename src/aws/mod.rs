use aws_config::{BehaviorVersion, SdkConfig};

pub mod dynamo;
pub mod s3;

pub async fn config() -> SdkConfig {
    aws_config::defaults(BehaviorVersion::latest()).load().await
}
