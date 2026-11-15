/*
 * Handling request to the server including signing, sending and receiving the result
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;

use crate::core::enums::RequestType;

#[async_trait]
pub trait Request {
    async fn sign_request(&self, payload: &str) -> Result<Box<str>, ArgusErr>;
    async fn send_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        request_type: RequestType,
        body: Option<&str>,
        headers: HeaderMap,
    ) -> Result<T, ArgusErr>;
}
