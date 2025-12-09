// 增加递归限制以解决 wgpu/burn 类型检查问题
#![recursion_limit = "1024"]

use std::sync::Arc;

use axum::{http::Method, Router};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod error;
mod handler;
mod model;

use model::ModelManager;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub model_manager: Arc<Mutex<ModelManager>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    info!("🚀 Starting LLM Inference Service...");
    info!("Multi-model support enabled");
    info!("Available models: Llama-3.1-8B, Llama-3.2-1B, Llama-3.2-3B");

    // 初始化模型管理器
    let model_manager = ModelManager::new().await?;
    info!("✅ Model loaded successfully!");

    let state = AppState {
        model_manager: Arc::new(Mutex::new(model_manager)),
    };

    // 配置CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        .merge(handler::routes())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // 启动服务器
    let addr = "0.0.0.0:8080";
    info!("🌐 Server listening on http://{}", addr);
    info!("");
    info!("📖 Available endpoints:");
    info!("   GET  /health              - Health check & current model");
    info!("   GET  /models              - List all available models");
    info!("   POST /models/switch       - Switch to a different model");
    info!("   POST /generate            - Text generation");
    info!("   POST /generate/stream     - Text generation (streaming)");
    info!("   POST /chat                - Chat completion");
    info!("   POST /chat/stream         - Chat completion (streaming)");
    info!("   GET  /v1/models           - OpenAI-compatible model list");
    info!("   POST /v1/completions      - OpenAI-compatible completions");
    info!("   POST /v1/chat/completions - OpenAI-compatible chat");
    info!("");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
