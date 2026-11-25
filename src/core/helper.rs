use argus::errors::ArgusErr;
use serde::Serialize;

pub async fn generate_request_body<P: Serialize + Send + Sync + 'static>(
    payload: P,
) -> Result<Box<str>, ArgusErr> {
    match serde_qs::to_string(&payload) {
        Ok(p) => Ok(Box::from(p)),
        Err(e) => Err(ArgusErr::operation("Failed to build request body!", e)),
    }
}
