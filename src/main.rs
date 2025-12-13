mod handler;
mod error;
mod types;
mod mistral_runner;
mod file_parser;
mod session;

use axum::{
    Router,
};
use axum::http::Method;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber;
use crate::file_parser::{new_file_cache, FileCache};
use crate::handler::routes;
use crate::session::{new_session_manager, SessionManager};

#[derive(Clone)]
pub struct AppState {
    pub file_cache: FileCache,
    pub session_manager: SessionManager,
}

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let state = AppState {
        file_cache: new_file_cache(),
        session_manager : new_session_manager(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}