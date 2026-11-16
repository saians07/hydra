use std::fmt::Display;

use argus::errors::ArgusErr;
use async_trait::async_trait;
use reqwest::{Client, header::HeaderMap};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;

use crate::{
    apis::{account::Account, request::Request},
    core::enums::{Hasher, RequestType},
};

#[derive(Debug, Clone)]
pub struct Binance {
    api_key: SecretString,
    secret_key: SecretString,
    client: Client,
    pub id: Box<str>,
    pub name: Box<str>,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Option<Box<str>>,
    pub public_ws_url: Option<Box<str>>,
    pub api_url_testing: Option<Box<str>>,
}

#[async_trait]
impl Account for Binance {
    async fn private_fetch_account_information<T: DeserializeOwned>(&self) -> Result<T, ArgusErr> {
        let query = format!("timestamp={}", chrono::Local::now().timestamp_millis());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );

        // account needs query to be signed
        let signed_query = self.sign_request(&query).await?;

        let endpoint = format!(
            "{}{}{}&signature={}",
            self.private_api_url, "/api/v3/account?", query, signed_query
        );

        self.send_request::<T>(&endpoint, RequestType::GET, None, headers)
            .await
    }
}

#[async_trait]
impl Request for Binance {
    async fn sign_request(&self, payload: &str) -> Result<Box<str>, ArgusErr> {
        let hasher = Hasher::Sha256;

        let signed_body = hasher.hash(payload, self.secret_key.expose_secret())?;

        Ok(Box::from(signed_body))
    }

    async fn send_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        request_type: RequestType,
        body: Option<&str>,
        headers: HeaderMap,
    ) -> Result<T, ArgusErr> {
        match request_type {
            RequestType::GET => {
                let request = self
                    .client
                    .get(endpoint)
                    .header("Accept", "application/json")
                    .headers(headers);

                request
                    .send()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to send request to the Binance server!", e)
                    })?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code", e))?
                    .json()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert Binance's response to json", e)
                    })
            }
            RequestType::POST => {
                let request = self
                    .client
                    .post(endpoint)
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .headers(headers)
                    .body(body.unwrap().to_owned());

                request
                    .send()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to post request to Binance server!", e)
                    })?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code", e))?
                    .json()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert Binance's response to json", e)
                    })
            }
            RequestType::DELETE => Err(ArgusErr::operation_ori(
                "DELETE request type is not yet supported!",
            )),
            RequestType::PATCH => Err(ArgusErr::operation_ori(
                "PATCH request type is not yet supported!",
            )),
            RequestType::PUT => Err(ArgusErr::operation_ori(
                "PUT request type is not yet supported!",
            )),
        }
    }
}

impl Display for Binance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The market is: {}", self.name)
    }
}

impl Binance {
    pub fn new(api_key: SecretString, secret_key: SecretString) -> Self {
        Self {
            id: "binance".into(),
            name: "Binance".into(),
            api_key,
            secret_key,
            client: Client::new(),
            private_api_url: "https://api.binance.com".into(),
            public_api_url: "https://api.binance.com".into(),
            private_ws_url: None,
            public_ws_url: None,
            api_url_testing: None,
        }
    }
}
