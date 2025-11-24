use crate::auth::Claims;
use crate::db::{SensorReading, UserForm};
use crate::http::{HttpResponse, LoginResponse, TimeRangeQuery};
use crate::solana::SolanaClient;
use crate::{auth, db};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router, middleware};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub client: Arc<SolanaClient>,
}

impl AppState {
    pub fn new(pool: PgPool, client: SolanaClient) -> Self {
        Self {
            pool,
            client: Arc::new(client),
        }
    }
}

// Define routes that require authentication
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/sensors/{sensor_id}/readings", get(fetch_reading))
        .route("/sensors", get(fetch_sensors))
        .layer(middleware::from_fn(auth::verify_jwt))
}

pub async fn root() -> &'static str {
    "Welcome to the Pollution Tracker API"
}

pub async fn db_health_check(State(state): State<AppState>) -> &'static str {
    match db::health_check(&state.pool).await {
        Ok(_) => "Database is up and running",
        Err(_) => "Database is down",
    }
}

pub async fn ingest_reading(
    State(state): State<AppState>,
    Json(payload): Json<SensorReading>,
) -> impl IntoResponse {
    // Validate payload: check for invalid values and missing fields
    if let Err(reason) = db::validate_reading(&payload) {
        return Json(HttpResponse::<()>::bad_request(reason)).into_response();
    }

    // Access control: check if sensor exists
    match db::sensor_exists(&state.pool, payload.sensor_id).await {
        Ok(exists) => {
            if exists == false {
                let reason = "Sensor is not registered";
                return Json(HttpResponse::<()>::bad_request(reason)).into_response();
            }
        }
        Err(e) => {
            println!("Error checking sensor existence: {}", e);
            return Json(HttpResponse::<()>::internal_error()).into_response();
        }
    }

    // Insert reading into DB
    if let Err(e) = db::insert_reading(&state.pool, &payload).await {
        println!("Error inserting reading: {}", e);
        return Json(HttpResponse::<()>::internal_error()).into_response();
    }

    // Submit proof to Solana blockchain
    match state.client.submit(payload).await {
        Ok(signature) => println!("Transaction submitted: {}", signature),
        Err(e) => {
            println!("Error submitting reading to Solana: {}", e);
            return Json(HttpResponse::<()>::internal_error()).into_response();
        }
    }

    Json(HttpResponse::<()>::success()).into_response()
}

pub async fn fetch_reading(
    sensor_id: Path<i32>,
    Query(range): Query<TimeRangeQuery>,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // Access control: check if user owns the sensor
    match db::owns_sensor(&state.pool, claims.sub.clone(), *sensor_id).await {
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

    match db::fetch_readings(&state.pool, *sensor_id, range, claims.sub).await {
        Ok(readings) => Json(HttpResponse::<_>::success_data(readings)).into_response(),
        Err(e) => {
            println!("Error fetching readings: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}

pub async fn fetch_sensors(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    match db::fetch_sensors(&state.pool, claims.sub).await {
        Ok(sensors) => Json(HttpResponse::<_>::success_data(sensors)).into_response(),
        Err(e) => {
            println!("Error fetching sensors: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}

pub async fn user_registry(
    State(state): State<AppState>,
    Json(form): Json<UserForm>,
) -> impl IntoResponse {
    match db::register_user(&state.pool, form).await {
        Ok(_) => Json(HttpResponse::<()>::success()).into_response(),
        Err(sqlx::Error::Database(e)) => {
            // PostgreSQL unique violation code
            if e.code() == Some(std::borrow::Cow::from("23505")) {
                println!("Username already taken");
                Json(HttpResponse::<()>::conflicts("Username already taken")).into_response()
            } else {
                println!("Error in user registry: {}", e);
                Json(HttpResponse::<()>::internal_error()).into_response()
            }
        }
        Err(e) => {
            println!("Error in user registry: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}

pub async fn user_login(
    State(state): State<AppState>,
    Json(form): Json<UserForm>,
) -> impl IntoResponse {
    match db::user_login(&state.pool, &form).await {
        Ok(valid) => {
            if valid {
                let token = auth::create_jwt(&form.username);
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
