mod db;
mod http;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use db::{SensorReading, insert_reading};
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
    if let Err(reason) = db::validate_reading(&payload) {
        return Json(ReadingResponse::bad_request(reason));
    }

    if let Err(e) = insert_reading(&pool, payload).await {
        println!("Error inserting reading: {}", e);
        return Json(ReadingResponse::internal_error());
    }

    Json(ReadingResponse::success())
}
