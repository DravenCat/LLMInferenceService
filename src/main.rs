mod handler;
mod model;
mod error;
mod tokenizer;

use std::sync::Arc;
use axum::{
    Router,
    http::method
};
use axum::http::Method;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber;
use model::ModelManager;


struct AppState {
    model_manager : Arc<Mutex<ModelManager>>,
}


#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let state = AppState {
        model_manager : Arc::new(Mutex::new(ModelManager::new("model.safetensors", "tokenizer.json").await))
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new();

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}