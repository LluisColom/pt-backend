use super::db::UserForm;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    range: Option<TimeRange>,
}

impl TimeRangeQuery {
    pub fn to_cutoff_time(&self) -> DateTime<Utc> {
        let now = Utc::now();

        let range = match self.range {
            None => return now - Duration::days(1), // Default is one day
            Some(range) => range,
        };

        match range {
            TimeRange::OneDay => now - Duration::days(1),
            TimeRange::OneWeek => now - Duration::weeks(1),
            TimeRange::OneMonth => now - Duration::days(30),
            TimeRange::OneQuarter => now - Duration::days(90),
            TimeRange::All => now - Duration::days(180), // Max 6 months
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum TimeRange {
    #[serde(rename = "24h")]
    OneDay,
    #[serde(rename = "7d")]
    OneWeek,
    #[serde(rename = "30d")]
    OneMonth,
    #[serde(rename = "90d")]
    OneQuarter,
    #[serde(rename = "all")]
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse<T>
where
    T: Serialize,
{
    status: u16,
    error_msg: Option<String>,
    body: Option<T>, // For success payloads
}

impl<T: Serialize> HttpResponse<T> {
    pub fn success() -> Self {
        HttpResponse {
            status: 200,
            error_msg: None,
            body: None,
        }
    }

    pub fn success_data(data: T) -> Self {
        HttpResponse {
            status: 200,
            error_msg: None,
            body: Some(data),
        }
    }

    pub fn bad_request(msg: impl AsRef<str>) -> Self {
        HttpResponse {
            status: 400,
            error_msg: Some(msg.as_ref().to_string()),
            body: None,
        }
    }

    pub fn unauthorized(msg: impl AsRef<str>) -> Self {
        HttpResponse {
            status: 401,
            error_msg: Some(msg.as_ref().to_string()),
            body: None,
        }
    }

    pub fn forbidden(msg: impl AsRef<str>) -> Self {
        HttpResponse {
            status: 403,
            error_msg: Some(msg.as_ref().to_string()),
            body: None,
        }
    }

    pub fn conflicts(msg: impl AsRef<str>) -> Self {
        HttpResponse {
            status: 409,
            error_msg: Some(msg.as_ref().to_string()),
            body: None,
        }
    }

    pub fn internal_error() -> Self {
        HttpResponse {
            status: 500,
            error_msg: Some("Internal server error".to_string()),
            body: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}

impl LoginResponse {
    pub fn new(token: String, user_form: &UserForm) -> Self {
        LoginResponse {
            token,
            username: user_form.username.clone(),
            role: "user".to_string(),
        }
    }
}
