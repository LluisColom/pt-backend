use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use chrono::{DateTime, Utc};
use dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Serialize, Deserialize)]
struct SensorReading {
    id: i32,
    sensor_id: i32,
    timestamp: DateTime<Utc>, // ISO 8601 format
    co2: f32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadingResponse {
    status: u16,
    error_msg: String,
}

impl ReadingResponse {
    fn success() -> Self {
        ReadingResponse {
            status: 200,
            error_msg: "".to_string(),
        }
    }

    fn bad_request(msg: impl AsRef<str>) -> Self {
        ReadingResponse {
            status: 400,
            error_msg: msg.as_ref().to_string(),
        }
    }

    fn internal_error() -> Self {
        ReadingResponse {
            status: 500,
            error_msg: "Internal server error".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Connect to database
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/db_health", get(db_health_check))
        .route("/reading", post(sensor_reading))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    // Run server
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn root() -> &'static str {
    "Welcome to the Pollution Tracker API"
}

async fn health_check() -> &'static str {
    "OK"
}

async fn db_health_check(State(pool): State<PgPool>) -> &'static str {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "OK",
        Err(_) => "FAILED",
    }
}

async fn sensor_reading(
    State(pool): State<PgPool>,
    Json(payload): Json<SensorReading>,
) -> Json<ReadingResponse> {
    if payload.co2 < 0.0 {
        return Json(ReadingResponse::bad_request("Invalid CO2 value"));
    }

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO readings (sensor_id, timestamp, co2_level)
        VALUES ($1, $2, $3)
        "#,
        payload.sensor_id,
        payload.timestamp,
        payload.co2
    )
    .execute(&pool)
    .await
    {
        println!("Error inserting reading: {}", e);
        return Json(ReadingResponse::internal_error());
    }

    Json(ReadingResponse::success())
}
