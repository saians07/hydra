use argus::errors::CustomErr;
use async_trait::async_trait;
use reqwest::{Client, Response};
use secrecy::SecretString;

use crate::utils::RequestType;

#[async_trait]
pub trait CexMarket {
    async fn build_payload<T: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: T,
    ) -> Result<Box<str>, CustomErr>;
    async fn sign_payload(
        &self,
        payload: &str,
        sign_key: SecretString,
    ) -> Result<Box<str>, CustomErr>;
    async fn send_request(
        &self,
        body: &str,
        url: &str,
        sign_key: Option<SecretString>,
        request_type: RequestType,
        client: Client,
    ) -> Result<Response, CustomErr>;
    async fn get_recv_window(&self) -> Result<(i64, i64), CustomErr>;
}
