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
