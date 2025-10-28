use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingResponse {
    status: u16,
    error_msg: String,
}

impl ReadingResponse {
    pub fn success() -> Self {
        ReadingResponse {
            status: 200,
            error_msg: "".to_string(),
        }
    }

    pub fn bad_request(msg: impl AsRef<str>) -> Self {
        ReadingResponse {
            status: 400,
            error_msg: msg.as_ref().to_string(),
        }
    }

    pub fn internal_error() -> Self {
        ReadingResponse {
            status: 500,
            error_msg: "Internal server error".to_string(),
        }
    }
}
