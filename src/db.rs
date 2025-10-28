use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await.map(|_| ())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReading {
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
        INSERT INTO readings (sensor_id, timestamp, co2_level)
        VALUES ($1, $2, $3)
        "#,
        payload.sensor_id,
        payload.timestamp,
        payload.co2
    )
    .execute(pool)
    .await?;

    println!("Inserted reading: {:?}", payload);
    Ok(())
}
