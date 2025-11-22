DROP DATABASE IF EXISTS pollution_tracker;
CREATE DATABASE pollution_tracker;

\connect pollution_tracker;

CREATE TABLE users (
   id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
   username TEXT UNIQUE NOT NULL,
   password TEXT NOT NULL,
   role TEXT DEFAULT 'user',
   created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE sensors (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(255),
    location VARCHAR(255),
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE readings (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    sensor_id INTEGER REFERENCES sensors(id),
    timestamp TIMESTAMPTZ NOT NULL,
    co2_level REAL NOT NULL,
    temperature REAL NOT NULL
);

-- Indexes to enhance performance
CREATE INDEX idx_sensor_readings_timestamp ON readings(sensor_id, timestamp);
CREATE INDEX idx_sensor_user_id ON sensors(user_id);

-- Insert test user
INSERT INTO users (username, password, role)
VALUES ('lluis', '$argon2id$v=19$m=19456,t=2,p=1$oz7KqblJDGMiSiaMPo9QyQ$QnuRj/P2YNuCUPdi2LrxBk/Kf85UwZj5qXxr6+zoJK8', 'user');

-- Insert test sensor
INSERT INTO sensors (sensor_name, sensor_location, user_id) VALUES ('Petroquimica Repsol', 'Tarragona', '1');