use super::crypto::{calculate_hash, verify_hash};
use super::http::TimeRangeQuery;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await.map(|_| ())
}

/// Model used to represent a sensor record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sensor {
    id: i32,
    name: String,
    location: String,
}

/// Model used to represent a sensor reading
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReading {
    pub(crate) sensor_id: i32,
    pub(crate) timestamp: DateTime<Utc>, // ISO 8601 format
    pub(crate) co2: f32,
    pub(crate) temperature: f32,
}

/// Model used to represent a sensor in the database
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

pub async fn insert_reading(pool: &PgPool, payload: &SensorReading) -> Result<(), sqlx::Error> {
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

pub async fn fetch_sensors(pool: &PgPool, username: String) -> Result<Vec<Sensor>, sqlx::Error> {
    // Read from DB
    let sensors = sqlx::query_as::<_, Sensor>(
        r#"
        SELECT
            s.id,
            s.name,
            s.location
        FROM sensors s
        INNER JOIN users u ON s.user_id = u.id
        WHERE u.username = $1
        ORDER BY s.name ASC
        "#,
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(sensors)
}

pub async fn fetch_readings(
    pool: &PgPool,
    sensor_id: i32,
    time_query: TimeRangeQuery,
    username: String,
) -> Result<Vec<SensorReadingRecord>, sqlx::Error> {
    // Extract DateTime from query
    let timestamp = time_query.to_cutoff_time();
    // Read from DB
    let readings = sqlx::query_as::<_, SensorReadingRecord>(
        r#"
        SELECT
            r.id,
            r.sensor_id,
            r.timestamp,
            r.co2_level as co2,
            r.temperature
        FROM readings r
        INNER JOIN sensors s ON r.sensor_id = s.id
        INNER JOIN users u ON s.user_id = u.id
        WHERE r.sensor_id = $1
        AND u.username = $2
        AND r.timestamp >= $3
        ORDER BY r.timestamp ASC
        "#,
    )
    .bind(sensor_id)
    .bind(username)
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
        SELECT password
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(&user_form.username)
    .fetch_optional(pool)
    .await?;

    Ok(stored_hash.map_or(false, |r| verify_hash(&user_form.password, &r.0)))
}

pub async fn sensor_exists(pool: &PgPool, sensor_id: i32) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM sensors WHERE id = $1) as "exists!"
        "#,
        sensor_id
    )
    .fetch_one(pool)
    .await?
    .exists;

    Ok(exists)
}

pub async fn owns_sensor(
    pool: &PgPool,
    username: String,
    sensor_id: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM sensors s
            INNER JOIN users u ON s.user_id = u.id
            WHERE s.id = $1 AND u.username = $2
        ) as "exists!"
        "#,
        sensor_id,
        username
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}
