use argus::errors::ArgusErr;

/// Convert any struct into string data to be the request body.
pub async fn generate_request_body<P: serde::Serialize + Send + Sync + 'static>(
    payload: P,
) -> Result<Box<str>, ArgusErr> {
    match serde_qs::to_string(&payload) {
        Ok(p) => return Ok(Box::from(p)),
        Err(e) => return Err(ArgusErr::operation("Failed to build request body!", e)),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct DummyStruct {
        name: String,
        age: i64,
    }

    #[tokio::test]
    async fn generate_request_body_test() {
        let body = DummyStruct {
            name: "test".to_string(),
            age: 64,
        };

        let res = generate_request_body(body).await;

        assert!(res.is_ok());

        let parsed = res.unwrap();
        assert_eq!(parsed, Box::from("name=test&age=64"))
    }
}
