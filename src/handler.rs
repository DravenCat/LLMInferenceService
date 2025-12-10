use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, GenerationConfig, ModelName, StreamChunk};
use crate::AppState;

// ============ 请求/响应结构 ============

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub current_model: String,
    pub available_models: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    /// 指定使用的模型（可选，不指定则使用当前加载的模型）
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    /// 是否将 prompt 包装为 chat 模板格式
    #[serde(default = "default_true")]
    pub use_chat_template: bool,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub token: String,
    pub generated_text: Option<String>,
    pub is_finished: bool,
    pub finish_reason: Option<String>,
}

fn default_true() -> bool {
    true
}

// ============ 路由配置 ============

pub fn routes() -> Router<AppState> {
    Router::new()
        // 模型管理
        .route("/health", get(health_check))
        // 文本生成
        .route("/generate/stream", post(generate_stream))
}

// ============ 模型管理端点 ============

/// 健康检查
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let model_manager = state.model_manager.lock().await;
    Json(HealthResponse {
        status: "ok".to_string(),
        current_model: model_manager.current_model().to_string(),
        available_models: ModelName::available_models().iter().map(|s| s.to_string()).collect(),
    })
}

// ============ 辅助函数：检查并切换模型 ============

async fn ensure_model(
    state: &AppState,
    requested_model: Option<&str>,
) -> AppResult<()> {
    if let Some(model_str) = requested_model {
        let model_name = ModelName::from_str(model_str)
            .ok_or_else(|| AppError::InvalidRequest(format!(
                "Unknown model '{}'. Available: {:?}",
                model_str,
                ModelName::available_models()
            )))?;
        
        let mut model_manager = state.model_manager.lock().await;
        if model_manager.current_model() != model_name {
            info!("Request requires model {}, switching...", model_name);
            model_manager.switch_model(model_name).await?;
        }
    }
    Ok(())
}

// ============ 生成端点 ============

/// 文本生成（流式）
pub async fn generate_stream(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Response {
    info!("Stream generate request: prompt length = {}", request.prompt.len());

    // 如果请求指定了模型，先切换
    if let Err(e) = ensure_model(&state, request.model.as_deref()).await {
        return Json(serde_json::json!({
            "error": e.to_string()
        })).into_response();
    }

    let config = build_config(&request);
    let model_manager = state.model_manager.lock().await;
    
    let prompt = if request.use_chat_template {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        }];
        model_manager.format_chat_prompt(&messages)
    } else {
        request.prompt.clone()
    };
    
    let rx = model_manager.stream(&prompt, Some(config)).await;
    drop(model_manager);

    let receiver_stream = ReceiverStream::new(rx);
    let event_stream = receiver_stream.map(|chunk: StreamChunk| {
        let response = StreamResponse {
            token: chunk.token_text,
            generated_text: if chunk.is_finished { Some(chunk.generated_text) } else { None },
            is_finished: chunk.is_finished,
            finish_reason: chunk.finish_reason,
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(json))
    });

    Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)).text("keep-alive"))
        .into_response()
}

// ============ 辅助函数 ============

fn build_config(request: &GenerateRequest) -> GenerationConfig {
    GenerationConfig {
        max_new_tokens: request.max_tokens.unwrap_or(256),
        temperature: request.temperature.unwrap_or(0.6),
        top_p: request.top_p.unwrap_or(0.9),
        seed: 42,
    }
}
