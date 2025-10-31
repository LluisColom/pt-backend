mod crypto;
mod db;
mod http;

use axum::extract::{Path, Query, State};
use axum::routing::post;
use axum::{Json, Router, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

use db::{SensorReading, SensorReadingRecord, UserForm};
use http::{ReadingResponse, TimeRangeQuery};

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

    // Allow requests from any origin (development-purposes only)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(db_health_check))
        .route("/sensors/ingest", post(ingest_reading))
        .route("/sensors/{sensor_id}/readings", get(fetch_reading))
        .route("/users/register", post(user_registry))
        .route("/users/login", post(user_login))
        .layer(cors)
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
    match db::health_check(&pool).await {
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

    if let Err(e) = db::insert_reading(&pool, payload).await {
        println!("Error inserting reading: {}", e);
        return Json(ReadingResponse::internal_error());
    }

    Json(ReadingResponse::success())
}

async fn fetch_reading(
    sensor_id: Path<i32>,
    Query(range): Query<TimeRangeQuery>,
    State(pool): State<PgPool>,
) -> Json<Vec<SensorReadingRecord>> {
    match db::fetch_readings(&pool, *sensor_id, range).await {
        Ok(readings) => Json(readings),
        Err(e) => {
            println!("Error fetching readings: {}", e);
            Json(vec![])
        }
    }
}

async fn user_registry(
    State(pool): State<PgPool>,
    Json(form): Json<UserForm>,
) -> Json<ReadingResponse> {
    if let Err(e) = db::register_user(&pool, form).await {
        println!("Error in user registry: {}", e);
    }
    Json(ReadingResponse::success())
}

async fn user_login(
    State(pool): State<PgPool>,
    Json(form): Json<UserForm>,
) -> Json<ReadingResponse> {
    match db::user_login(&pool, form).await {
        Ok(valid) => {}
        Err(e) => {
            println!("Error in user login: {}", e);
        }
    }
    Json(ReadingResponse::success())
}
