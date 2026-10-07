/*
 * This will handle request sent to the server.
 * In sending request, servers provide various type of
 * ways. But most of the time, they will be about:
 *  - building payload
 *  - signing payload
 *  - sending get request
 *  - sending post request
 */
use anyhow::Result;
use argus::errors::ArgusErr;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;

use crate::core::utils::RequestType;

#[async_trait]
pub trait Request {
    /// Sign data to authorize access to private endpoint API
    ///
    /// Return signed data as Box<str>
    ///
    /// # Arguments
    ///
    /// * `payload`: Final data sent as body to the server.
    ///
    /// # Errors
    ///
    /// Will return `Err(ArgusErr)` if the signing process failed.
    async fn sign_request(&self, payload: &str) -> Result<Box<str>, ArgusErr>;
    /// Send the request to the server. This will accomodate both get and post request type
    ///
    /// Some server might request body, some server might request query, where
    /// each query and body should be handled uniquely for each server.
    ///
    /// # Arguments
    ///
    /// - `base_url` - The endpoint url provides the functionality from the server.
    /// - `request_type` - Defines whether the request is a GET or a POST
    /// - `body` - The body needs to be sent along with the request.
    /// - `query` - The query needs to be sent with the request.
    async fn send_request<T: DeserializeOwned>(
        &self,
        base_url: &str,
        request_type: RequestType,
        body: Option<&str>,
        query: Option<&str>,
        headers: HeaderMap,
        need_signed: bool,
    ) -> Result<T, ArgusErr>;
}
