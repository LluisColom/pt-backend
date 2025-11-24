mod api;
mod auth;
mod crypto;
mod db;
mod http;
mod solana;

use axum::routing::post;
use axum::{Router, routing::get};
use solana::SolanaClient;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _ = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Initialize Solana client
    let rpc_url = std::env::var("SOLANA_RPC").expect("RPC url must be set");
    let keypair = std::env::var("SOLANA_KEYPAIR").expect("Solana keypair must be set");
    let client = SolanaClient::new(&rpc_url, &keypair)?;
    client.test_connection().await?;
    anyhow::ensure!(client.enough_balance()?, "Insufficient balance");

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db)
        .await
        .expect("Failed to connect to database");

    let app_state = api::AppState::new(pool, client);

    // Allow requests from any origin (development-purposes only)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(api::root))
        .route("/health", get(api::db_health_check))
        .route("/users/register", post(api::user_registry))
        .route("/users/login", post(api::user_login))
        .route("/sensors/ingest", post(api::ingest_reading))
        // Merge protected routes as a separate router
        .merge(api::protected_routes())
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    // Run server
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
