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
- HTTPS/TLS support with rustls

### Blockchain Features
- Solana memo program integration for on-chain data storage
- Low-cost transactions (~0.000005 SOL per reading)

## Prerequisites

- Rust 1.83 or higher
- PostgreSQL 16 or higher
- Solana CLI tools (optional, for manual blockchain interaction)
- OpenSSL or compatible (for TLS certificates)

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
SOLANA_RPC=https://api.devnet.solana.com
SOLANA_KEYPAIR=./solana-keypair.json
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

**Production Mode with HTTPS** (requires certificates):
```bash
cargo build --release
# Generate self-signed certificates (development)
mkcert -install
mkcert localhost 127.0.0.1 ::1

# Run server
cargo run --release
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

## Configuration

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

## Future Improvements
- [ ] Docker containerization + Kubernetes
- [ ] Rate limiting per IP and per user
- [ ] API key authentication for sensors
- [ ] Database column-level encryption
- [ ] Audit logging for sensitive operations
- [ ] DDoS protection
- [ ] Content Security Policy headers

## License

This project is part of academic coursework for MSc in Cybersecurity at UPC Barcelona. MIT License.

## Author

Lluís Colom - MSc Cybersecurity, UPC Barcelona

## Contributing

This is an academic project. For questions or improvements, please contact the author.
