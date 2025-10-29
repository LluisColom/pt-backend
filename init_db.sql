DROP DATABASE IF EXISTS pollution_tracker;
CREATE DATABASE pollution_tracker;

\connect pollution_tracker;

CREATE TABLE sensors (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    sensor_name VARCHAR(255),
    sensor_location VARCHAR(255)
);

CREATE TABLE readings (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    sensor_id INTEGER REFERENCES sensors(id),
    timestamp TIMESTAMPTZ NOT NULL,
    co2_level REAL NOT NULL,
    temperature REAL NOT NULL
);

INSERT INTO sensors (sensor_name, sensor_location) VALUES ('Petroquimica Repsol', 'Tarragona');