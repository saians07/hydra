use argus::errors::CustomErr;
use async_trait::async_trait;
use hydra_core::{
    market::crypto::CexMarket,
    utils::{RequestType, hmac_512},
};
use reqwest::{Client, Response, header::HeaderValue};
use secrecy::{ExposeSecret, SecretString};

pub struct Indodax {
    pub id: Box<str>,
    pub name: Box<str>,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Box<str>,
    pub public_ws_url: Box<str>,
}

impl Indodax {
    pub fn new(id: Box<str>, name: Box<str>) -> Self {
        Self {
            id,
            name,
            private_api_url: "https://indodax.com/tapi".into(),
            public_api_url: "https://indodax.com".into(),
            private_ws_url: "".into(),
            public_ws_url: "".into(),
        }
    }
}

#[async_trait]
impl CexMarket for Indodax {
    /// Serialize any struct to build the payload data.
    async fn build_payload<P: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: P,
    ) -> Result<Box<str>, CustomErr> {
        match serde_qs::to_string(&payload) {
            Ok(payload) => return Ok(Box::from(payload)),
            Err(e) => return Err(CustomErr::operation("Failed to build payload:", e)),
        }
    }

    async fn sign_payload(
        &self,
        payload: &str,
        sign_key: SecretString,
    ) -> Result<Box<str>, CustomErr> {
        let result = hmac_512(payload, sign_key.expose_secret())?;

        Ok(Box::from(result))
    }

    async fn send_request(
        &self,
        body: &str,
        url: &str,
        sign_key: Option<SecretString>,
        request_type: RequestType,
        client: Client,
    ) -> Result<Response, CustomErr> {
        match request_type {
            RequestType::GET => {
                let result = client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| CustomErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| CustomErr::operation("Error with status code:", e))?;

                Ok(result)
            }
            RequestType::POST => {
                let Some(sign_key) = sign_key else {
                    return Err(CustomErr::Operation {
                        operation: "Failed to send request to Indodax".into(),
                        source: None,
                    });
                };
                let signed_body = self.sign_payload(body, sign_key.clone()).await?;
                let result = client
                    .post(url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header(
                        "Key",
                        HeaderValue::from_str(&sign_key.clone().expose_secret()).unwrap(),
                    )
                    .header("Sign", HeaderValue::from_str(&signed_body).unwrap())
                    .send()
                    .await
                    .map_err(|e| CustomErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| CustomErr::operation("Error with status code:", e))?;

                Ok(result)
            }
        }
    }

    async fn get_recv_window(&self) -> Result<(i64, i64), CustomErr> {
        let now = chrono::Local::now().timestamp_millis();

        // this will be deduced with how long the request is valid for
        let recv_window = now - 4000;

        Ok((now, recv_window))
    }
}

// this is the specific implementation of Indodax endpoint.
impl Indodax {
    /// Private getInfo endpoint of Indondax
    pub async fn priv_get_info(
        &self,
        sign_key: SecretString,
        client: Client,
    ) -> Result<Response, CustomErr> {
        let (now, recv_window) = self.get_recv_window().await?;
        let body = format!(
            "method=getInfo&timestamp={:?}&recvWindow={:?}",
            now, recv_window
        );

        let result = self
            .send_request(
                &body,
                &self.private_api_url,
                Some(sign_key),
                RequestType::POST,
                client,
            )
            .await?;

        Ok(result)
    }
}
