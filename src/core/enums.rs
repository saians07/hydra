use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}
