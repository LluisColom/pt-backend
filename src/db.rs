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
) -> Result<Vec<SensorReadingRecord>, sqlx::Error> {
    let readings = sqlx::query_as::<_, SensorReadingRecord>(
        r#"
        SELECT id, sensor_id, timestamp, co2_level as co2, temperature
        FROM readings
        WHERE sensor_id = $1
        ORDER BY timestamp ASC
        "#,
    )
    .bind(sensor_id)
    .fetch_all(pool)
    .await?;

    Ok(readings)
}
