pub mod api;
pub mod web;

use axum::{
    routing::{get, post},
    Router,
    http::{StatusCode, Method},
    response::Json,
};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use anyhow::Result;

pub async fn start_server(port: u16) -> Result<()> {
    let app = create_app();
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    
    println!("Server running on http://127.0.0.1:{}", port);
    println!("API endpoints:");
    println!("  GET  /api/health - Health check");
    println!("  POST /api/analyze - Analyze codebase");
    println!("  GET  /api/integrations - Integration status");
    println!("  GET  / - Web interface");
    
    axum::serve(listener, app).await?;
    Ok(())
}

fn create_app() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/", get(web::serve_index))
        .route("/api/health", get(api::health_check))
        .route("/api/analyze", post(api::analyze_codebase))
        .route("/api/integrations", get(api::integration_status))
        .layer(ServiceBuilder::new().layer(cors))
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}