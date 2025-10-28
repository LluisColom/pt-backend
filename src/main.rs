use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    println!("Starting webserver");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check));

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
