use super::crypto::{calculate_hash, verify_hash};
use super::http::TimeRangeQuery;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await.map(|_| ())
}

/// Model used to represent a sensor reading request
/// Used for validation and insertion
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReading {
    sensor_id: i32,
    timestamp: DateTime<Utc>, // ISO 8601 format
    co2: f32,
    temperature: f32,
}

/// Model used to represent a sensor reading record
/// Used for fetching and returning data
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SensorReadingRecord {
    id: i32,
    sensor_id: i32,
    timestamp: DateTime<Utc>, // ISO 8601 format
    co2: f32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct UserForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, FromRow)]
struct UserRecord(String); // Tuple struct

pub fn validate_reading(payload: &SensorReading) -> Result<(), &'static str> {
    if payload.co2 < 0.0 {
        return Err("Invalid CO2 value");
    }
    // TODO - validate the other fields
    Ok(())
}

pub async fn insert_reading(pool: &PgPool, payload: SensorReading) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO readings (sensor_id, timestamp, co2_level, temperature)
        VALUES ($1, $2, $3, $4)
        "#,
        payload.sensor_id,
        payload.timestamp,
        payload.co2,
        payload.temperature
    )
    .execute(pool)
    .await?;

    println!("Inserted reading: {:?}", payload);
    Ok(())
}

pub async fn fetch_readings(
    pool: &PgPool,
    sensor_id: i32,
    time_query: TimeRangeQuery,
) -> Result<Vec<SensorReadingRecord>, sqlx::Error> {
    // Extract DateTime from query
    let timestamp = time_query.to_cutoff_time();
    // Read from DB
    let readings = sqlx::query_as::<_, SensorReadingRecord>(
        r#"
        SELECT id, sensor_id, timestamp, co2_level as co2, temperature
        FROM readings
        WHERE sensor_id = $1
        AND timestamp >= $2
        ORDER BY timestamp ASC
        "#,
    )
    .bind(sensor_id)
    .bind(timestamp)
    .fetch_all(pool)
    .await?;

    Ok(readings)
}

pub async fn register_user(pool: &PgPool, user_form: UserForm) -> Result<(), sqlx::Error> {
    // Calculate Argon2 password hash
    let hash = calculate_hash(user_form.password.as_str());
    // Insert into DB
    sqlx::query!(
        r#"
        INSERT INTO users (username, password)
        VALUES ($1, $2)
        "#,
        user_form.username,
        hash
    )
    .execute(pool)
    .await?;

    println!("New user created: {}", user_form.username);
    Ok(())
}

pub async fn user_login(pool: &PgPool, user_form: &UserForm) -> Result<bool, sqlx::Error> {
    // Read stored hash from DB
    let stored_hash = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT password_hash
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(&user_form.username)
    .fetch_optional(pool)
    .await?;

    Ok(stored_hash.map_or(false, |r| verify_hash(&user_form.password, &r.0)))
}
