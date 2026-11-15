use reqwest::Client;
use secrecy::SecretString;

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
