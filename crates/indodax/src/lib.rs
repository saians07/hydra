use argus::errors::CustomErr;
use async_trait::async_trait;
use hydra_core::{
    balance::BalanceType,
    utils::{RequestType, hmac_512},
};
use reqwest::{Client, Response, header::HeaderValue};

pub struct Indodax {
    pub id: Box<str>,
    pub name: Box<str>,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Box<str>,
    pub public_ws_url: Box<str>,
    pub balance: BalanceType,
}

#[async_trait]
impl hydra_core::market::CryptoCexMarket for Indodax {
    /// Build payload from any kind of srtucture data to match
    /// the API query.
    async fn build_payload<T: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: T,
    ) -> Result<Box<str>, CustomErr> {
        match serde_qs::to_string(&payload) {
            Ok(payload) => return Ok(Box::from(payload)),
            Err(e) => return Err(CustomErr::operation("Failed to build payload:", e)),
        }
    }

    async fn sign_payload(&self, payload: &str, key: &str) -> Result<Box<str>, CustomErr> {
        let result = hmac_512(payload, key)?;

        Ok(Box::from(result))
    }

    async fn send_request(
        &self,
        body: &str,
        url: &str,
        sign_key: Option<&str>,
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
                let signed_body = self.sign_payload(body, sign_key).await?;
                let result = client
                    .post(url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Key", HeaderValue::from_str(sign_key).unwrap())
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
}
