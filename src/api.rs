use crate::auth::Claims;
use crate::db::{SensorReading, UserForm};
use crate::http::{HttpResponse, LoginResponse, TimeRangeQuery};
use crate::{auth, db};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router, middleware};
use sqlx::PgPool;

// Define routes that require authentication
pub fn protected_routes() -> Router<PgPool> {
    Router::new()
        .route("/sensors/{sensor_id}/readings", get(fetch_reading))
        .route("/sensors", get(fetch_sensors))
        .layer(middleware::from_fn(auth::verify_jwt))
}

pub async fn root() -> &'static str {
    "Welcome to the Pollution Tracker API"
}

pub async fn db_health_check(State(pool): State<PgPool>) -> &'static str {
    match db::health_check(&pool).await {
        Ok(_) => "Database is up and running",
        Err(_) => "Database is down",
    }
}

pub async fn ingest_reading(
    State(pool): State<PgPool>,
    Json(payload): Json<SensorReading>,
) -> impl IntoResponse {
    if let Err(reason) = db::validate_reading(&payload) {
        return Json(HttpResponse::<()>::bad_request(reason)).into_response();
    }

    match db::sensor_exists(&pool, payload.sensor_id).await {
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

    if let Err(e) = db::insert_reading(&pool, payload).await {
        println!("Error inserting reading: {}", e);
        return Json(HttpResponse::<()>::internal_error()).into_response();
    }

    Json(HttpResponse::<()>::success()).into_response()
}

pub async fn fetch_reading(
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

pub async fn fetch_sensors(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    match db::fetch_sensors(&pool, claims.sub).await {
        Ok(sensors) => Json(HttpResponse::<_>::success_data(sensors)).into_response(),
        Err(e) => {
            println!("Error fetching sensors: {}", e);
            Json(HttpResponse::<()>::internal_error()).into_response()
        }
    }
}

pub async fn user_registry(
    State(pool): State<PgPool>,
    Json(form): Json<UserForm>,
) -> impl IntoResponse {
    match db::register_user(&pool, form).await {
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
    State(pool): State<PgPool>,
    Json(form): Json<UserForm>,
) -> impl IntoResponse {
    match db::user_login(&pool, &form).await {
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
