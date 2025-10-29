mod db;
mod http;

use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router, routing::get};
use dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use db::{SensorReading, SensorReadingRecord, fetch_readings, health_check, insert_reading};
use http::ReadingResponse;

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
        .route("/health", get(db_health_check))
        .route("/sensors/ingest", post(ingest_reading))
        .route("/sensors/{sensor_id}/readings", get(fetch_reading))
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

async fn db_health_check(State(pool): State<PgPool>) -> &'static str {
    match health_check(&pool).await {
        Ok(_) => "Database is up and running",
        Err(_) => "Database is down",
    }
}

async fn ingest_reading(
    State(pool): State<PgPool>,
    Json(payload): Json<SensorReading>,
) -> Json<ReadingResponse> {
    if let Err(reason) = db::validate_reading(&payload) {
        return Json(ReadingResponse::bad_request(reason));
    }

    if let Err(e) = insert_reading(&pool, payload).await {
        println!("Error inserting reading: {}", e);
        return Json(ReadingResponse::internal_error());
    }

    Json(ReadingResponse::success())
}

async fn fetch_reading(
    State(pool): State<PgPool>,
    sensor_id: Path<i32>,
) -> Json<Vec<SensorReadingRecord>> {
    match fetch_readings(&pool, *sensor_id).await {
        Ok(readings) => Json(readings),
        Err(e) => {
            println!("Error fetching readings: {}", e);
            Json(vec![])
        }
    }
}
