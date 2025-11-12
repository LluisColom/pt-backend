mod auth;
mod crypto;
mod db;
mod http;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Extension, Json, Router, middleware, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

use auth::Claims;
use db::{SensorReading, UserForm};
use http::{HttpResponse, LoginResponse, TimeRangeQuery};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _ = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Connect to database
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
        .route("/users/register", post(user_registry))
        .route("/users/login", post(user_login))
        .route("/sensors/ingest", post(ingest_reading))
        // Merge protected routes as a separate router
        .merge(protected_routes())
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

// Define routes that require authentication
fn protected_routes() -> Router<PgPool> {
    Router::new()
        .route("/sensors/{sensor_id}/readings", get(fetch_reading))
        .layer(middleware::from_fn(auth::verify_jwt))
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
) -> impl IntoResponse {
    if let Err(reason) = db::validate_reading(&payload) {
        return Json(HttpResponse::<()>::bad_request(reason)).into_response();
    }

    if let Err(e) = db::insert_reading(&pool, payload).await {
        println!("Error inserting reading: {}", e);
        return Json(HttpResponse::<()>::internal_error()).into_response();
    }

    Json(HttpResponse::<()>::success()).into_response()
}

async fn fetch_reading(
    sensor_id: Path<i32>,
    Query(range): Query<TimeRangeQuery>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // Access control: check if user owns the sensor
    match db::owns_sensor(&pool, claims.sub.clone(), *sensor_id).await {
        Ok(ownership) => {
            if ownership == false {
                let msg = "Not authorized to access this sensor";
                return Json(HttpResponse::<()>::forbidden(msg)).into_response();
            }
        }
        Err(e) => {
            println!("Database error checking ownership: {}", e);
            return Json(HttpResponse::<()>::internal_error()).into_response();
        }
    }

    match db::fetch_readings(&pool, *sensor_id, range, claims.sub).await {
        Ok(readings) => Json(HttpResponse::<_>::success_data(readings)).into_response(),
        Err(e) => {
            println!("Error fetching readings: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}

async fn user_registry(
    State(pool): State<PgPool>,
    Json(form): Json<UserForm>,
) -> impl IntoResponse {
    if let Err(e) = db::register_user(&pool, form).await {
        println!("Error in user registry: {}", e);
        return Json(HttpResponse::<()>::internal_error()).into_response();
    }
    Json(HttpResponse::<()>::success()).into_response()
}

async fn user_login(State(pool): State<PgPool>, Json(form): Json<UserForm>) -> impl IntoResponse {
    match db::user_login(&pool, &form).await {
        Ok(valid) => {
            if valid {
                let token = auth::create_jwt(&form);
                let resp = LoginResponse::new(token, &form);
                Json(HttpResponse::success_data(resp)).into_response()
            } else {
                Json(HttpResponse::<()>::unauthorized("Invalid credentials")).into_response()
            }
        }
        Err(e) => {
            println!("Error in user login: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}
