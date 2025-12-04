# Pollution Tracker - Backend

A high-performance, security-first Rust backend for the Pollution Tracker web application. Built with Axum framework, PostgreSQL, and Solana blockchain integration for cryptographically-verified pollution monitoring. Developed as part of the MSc in Cybersecurity program at UPC Barcelona.

## Overview

The Pollution Tracker backend provides a RESTful API for managing pollution sensor data with blockchain-backed integrity verification. The application demonstrates secure-by-design principles with comprehensive authentication, authorization, data validation, and cryptographic proof generation using Solana devnet.

## Features

### Core Functionality
- **JWT Authentication**: Secure token-based authentication with Argon2 password hashing
- **Sensor Management**: CRUD operations for pollution sensors with user ownership
- **Data Ingestion**: High-performance sensor reading ingestion with validation
- **Time-Range Queries**: Efficient PostgreSQL queries with date filtering (24h, 7d, 30d)
- **Blockchain Integration**: Automatic hash generation and Solana devnet transaction submission
- **Data Verification**: Cryptographic verification against blockchain proofs
- **CORS Support**: Configurable cross-origin resource sharing for frontend integration

### Security Features
- Argon2 password hashing with secure salt generation
- JWT token generation with expiration (HS256 algorithm)
- Middleware-based authentication for protected routes
- User-sensor authorization (users can only access their own sensors)
- SQL injection prevention through parameterized queries
- Input validation and sanitization
- Secure environment variable management
- HTTPS/TLS support with rustls
- Rate limiting ready (can be implemented)

### Blockchain Features
- Blake3 hash generation for sensor readings
- Solana memo program integration for on-chain data storage
- Transaction signature tracking in database
- Deterministic verification system
- Immutable audit trail via blockchain
- Low-cost transactions (~0.000005 SOL per reading)

## Technology Stack

### Core Technologies
- **Rust 1.83+**: Systems programming language with memory safety
- **Axum 0.8**: Modern, ergonomic web framework built on Tokio
- **Tokio 1.48**: Async runtime for concurrent request handling
- **PostgreSQL 16**: Relational database with ACID guarantees
- **SQLx 0.8**: Compile-time checked SQL queries

### Key Libraries
- **solana-client 3.1**: Solana RPC client for blockchain interaction
- **solana-sdk 3.0**: Solana transaction building and signing
- **spl-memo 6.0**: Solana memo program for on-chain data storage
- **jsonwebtoken 10.1**: JWT token creation and validation
- **argon2 0.5**: Password hashing algorithm
- **blake3 1.8**: Fast cryptographic hashing
- **chrono 0.4**: Date and time handling with timezone support
- **tower-http 0.6**: HTTP middleware (CORS, compression, etc.)
- **anyhow 1.0**: Error handling and propagation
- **dotenv 0.15**: Environment variable loading
- **rustls 0.23**: Pure Rust TLS implementation

## Prerequisites

- Rust 1.83 or higher
- PostgreSQL 16 or higher
- Solana CLI tools (optional, for manual blockchain interaction)
- OpenSSL or compatible (for TLS certificates)
- macOS/Linux/Windows with modern CPU

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/LluisColom/pt-backend.git
cd pt-backend
```

### 2. Install Rust Dependencies

```bash
cargo build --release
```

This compiles the project with optimizations enabled.

### 3. Set Up PostgreSQL Database

**Install PostgreSQL** (if not already installed):
```bash
# macOS
brew install postgresql@16
brew services start postgresql@16
```

**Initialize Database**:
```bash
# Create database and tables
psql -f init_db.sql postgres
```

### 4. Configure Environment Variables

Create a `.env` file in the project root:

```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost/pollution_tracker

# JWT Secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-super-secret-jwt-key-minimum-32-characters-long-random

# Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_KEYPAIR_PATH=./solana-keypair.json
```

### 5. Generate Solana Keypair using Solana CLI

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Generate keypair
solana-keygen new --outfile ./solana-keypair.json --no-bip39-passphrase

# Get devnet SOL (airdrop)
solana airdrop 2 $(solana-keygen pubkey ./solana-keypair.json) --url devnet
```

## Usage

### Running the Server

**Development Mode** (with debug logging):
```bash
RUST_LOG=debug cargo run
```

**Production Mode** (optimized build):
```bash
cargo build --release
./target/release/PollutionTracker
```

**With HTTPS** (requires certificates):
```bash
# Generate self-signed certificates (development)
mkcert -install
mkcert localhost 127.0.0.1 ::1

# Run server
cargo run --release
```

The server will start on:
- HTTP: `http://127.0.0.1:3000`
- HTTPS: `https://127.0.0.1:3000`

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_user_registration

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

### Database Management

**Create Test User**:
```bash
# Using utility binary
cargo run --bin hash_password "testpassword123"
# Copy the hash output

# Insert into database
psql -d pollution_tracker
INSERT INTO users (username, password_hash, role) 
VALUES ('testuser', '$argon2id$v=19$m=19456...', 'user');
```

**Query Database**:
```bash
# Connect to database
psql -d pollution_tracker

# Common queries
SELECT * FROM users;
SELECT * FROM sensors WHERE user_id = 1;
SELECT * FROM readings WHERE sensor_id = 1 ORDER BY timestamp DESC LIMIT 10;
SELECT COUNT(*) FROM readings WHERE blockchain_tx IS NOT NULL;
```

### Blockchain Operations

**Check Solana Connection**:
```bash
# View public key
solana-keygen pubkey ./solana-keypair.json

# Check balance
solana balance $(solana-keygen pubkey ./solana-keypair.json) --url devnet

# View recent transactions
solana transaction-history $(solana-keygen pubkey ./solana-keypair.json) --url devnet
```

## Blockchain Integration

### Hash Generation

**Algorithm**: Blake3

**Input Format**:
```
sensor:{sensor_id}|ts:{unix_timestamp}|co2:{co2_value}|temp:{temperature_value}
```

**Example**:
```
Input: sensor:1|ts:1733310645|co2:95000.0|temp:82.5
Output: a3f5d8e2b1c4f7a9e3d6c8b5f2a1d4e7c9b6a3f5d8e2b1c4f7a9e3d6c8b5f2a1
```

### Memo Format

**Structure**:
```
pollution|v1|s:{sensor_id}|h:{hash_prefix}|ts:{timestamp}
```

**Example**:
```
pollution|v1|s:1|h:a3f5d8e2b1c4f7a9|ts:1733310645
```

**Fields**:
- `pollution`: Application identifier
- `v1`: Format version for future compatibility
- `s:{sensor_id}`: Sensor identifier
- `h:{hash_prefix}`: First 16 characters of hash
- `ts:{timestamp}`: Unix timestamp

**Full hash stored in database for complete verification.**

### Transaction Workflow

1. **Receive Reading**: POST /sensors/ingest with sensor data
2. **Generate Hash**: Compute SHA-256 of formatted reading
3. **Create Memo**: Format memo with hash and metadata
4. **Build Transaction**:
   - Get latest blockhash from Solana RPC
   - Create memo instruction with formatted data
   - Sign transaction with keypair
5. **Submit Transaction**: Send to Solana devnet
6. **Store Proof**: Save transaction signature in database
7. **Return Response**: Include reading ID and blockchain TX

### Verification Process

1. **Fetch Reading**: Query database for reading by ID
2. **Recalculate Hash**: Generate hash from current database values
3. **Fetch Transaction**: Query Solana blockchain for transaction
4. **Extract Memo**: Parse memo field from transaction
5. **Compare Hashes**: Verify database hash matches blockchain hash
6. **Validate Timestamp**: Ensure blockchain timestamp matches reading
7. **Return Result**: Verified (true/false) with proof details

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `JWT_SECRET` | Yes | - | Secret key for JWT signing (min 32 chars) |
| `SOLANA_RPC_URL` | No | `https://api.devnet.solana.com` | Solana RPC endpoint |
| `SOLANA_KEYPAIR_PATH` | No | `./solana-keypair.json` | Path to Solana keypair file |
| `HOST` | No | `127.0.0.1` | Server bind address |
| `PORT` | No | `3000` | Server port |
| `RUST_LOG` | No | `info` | Logging level (error/warn/info/debug/trace) |

### CORS Configuration

**Allowed Origins**:
```rust
// In main.rs
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

**Production**: Update to your production frontend URL.

### TLS Configuration

**Certificate Paths**:
```rust
let config = RustlsConfig::from_pem_file(
    "localhost+2.pem",      // Certificate
    "localhost+2-key.pem"   // Private key
).await?;
```

## Performance Considerations

### Current Optimizations
- Async/await throughout for non-blocking I/O
- Connection pooling with SQLx (max 5 connections)
- Indexed database queries on sensor_id and timestamp
- Compiled queries with SQLx macros (compile-time validation)
- Release builds with optimizations enabled
- Efficient JSON serialization with serde

## Future Improvements
- [ ] Docker containerization + Kubernetes
- [ ] Rate limiting per IP and per user
- [ ] API key authentication for sensors
- [ ] Database column-level encryption
- [ ] Audit logging for sensitive operations
- [ ] DDoS protection
- [ ] Content Security Policy headers

## Deployment

### Production Checklist

- [ ] Change JWT_SECRET to strong random value
- [ ] Update DATABASE_URL to production database
- [ ] Configure production CORS origins
- [ ] Set up HTTPS with valid TLS certificates
- [ ] Enable rate limiting
- [ ] Configure database backups
- [ ] Set up CI/CD pipeline
- [ ] Configure firewall rules
- [ ] Enable database connection pooling

## License

This project is part of academic coursework for MSc in Cybersecurity at UPC Barcelona. MIT License.

## Author

Lluís Colom - MSc Cybersecurity, UPC Barcelona

## Contributing

This is an academic project. For questions or improvements, please contact the author.
