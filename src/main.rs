use axum::extract::State;
use axum::{Router, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Connect to database
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/db_health", get(db_health_check))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    // Run server
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn root() -> &'static str {
    "Welcome to the Pollution Tracker API"
}

async fn health_check() -> &'static str {
    "OK"
}

async fn db_health_check(State(pool): State<PgPool>) -> &'static str {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "OK",
        Err(_) => "FAILED",
    }
}
