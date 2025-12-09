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
use crate::model::{ChatMessage, GenerationConfig, ModelInfo, ModelManager, ModelName, StreamChunk};
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
    #[serde(default)]
    pub stream: Option<bool>,
    /// 是否将 prompt 包装为 chat 模板格式
    #[serde(default = "default_true")]
    pub use_chat_template: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    /// 指定使用的模型（可选）
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchModelRequest {
    /// 要切换到的模型名称
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SwitchModelResponse {
    pub success: bool,
    pub message: String,
    pub current_model: String,
}

#[derive(Debug, Serialize)]
pub struct ListModelsResponse {
    pub models: Vec<ModelInfo>,
    pub current_model: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub text: String,
    pub model: String,
    pub usage: UsageInfo,
    pub inference_time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub usage: UsageInfo,
    pub inference_time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct UsageInfo {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
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
        .route("/models", get(list_models))
        .route("/models/switch", post(switch_model))
        // 文本生成
        .route("/generate", post(generate))
        .route("/generate/stream", post(generate_stream))
        // 聊天
        .route("/chat", post(chat))
        .route("/chat/stream", post(chat_stream))
        // OpenAI 兼容
        .route("/v1/models", get(list_models_openai))
        .route("/v1/completions", post(generate_completion))
        .route("/v1/chat/completions", post(chat_completion))
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

/// 列出所有可用模型
pub async fn list_models(State(state): State<AppState>) -> Json<ListModelsResponse> {
    let model_manager = state.model_manager.lock().await;
    Json(ListModelsResponse {
        models: model_manager.list_models(),
        current_model: model_manager.current_model().to_string(),
    })
}

/// OpenAI 兼容的模型列表
pub async fn list_models_openai(State(state): State<AppState>) -> Json<serde_json::Value> {
    let model_manager = state.model_manager.lock().await;
    let models: Vec<serde_json::Value> = model_manager.list_models().iter().map(|m| {
        serde_json::json!({
            "id": m.name,
            "object": "model",
            "owned_by": "meta",
            "permission": []
        })
    }).collect();
    
    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

/// 切换模型
pub async fn switch_model(
    State(state): State<AppState>,
    Json(request): Json<SwitchModelRequest>,
) -> AppResult<Json<SwitchModelResponse>> {
    info!("Switch model request: {}", request.name);
    
    let model_name = ModelName::from_str(&request.name)
        .ok_or_else(|| AppError::InvalidRequest(format!(
            "Unknown model '{}'. Available models: {:?}",
            request.name,
            ModelName::available_models()
        )))?;
    
    let mut model_manager = state.model_manager.lock().await;
    model_manager.switch_model(model_name).await?;
    
    Ok(Json(SwitchModelResponse {
        success: true,
        message: format!("Successfully switched to {}", model_name),
        current_model: model_name.to_string(),
    }))
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

/// 文本生成（非流式）
pub async fn generate(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> AppResult<Json<GenerateResponse>> {
    info!("Generate request: prompt length = {}", request.prompt.len());

    // 如果请求指定了模型，先切换
    ensure_model(&state, request.model.as_deref()).await?;

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
    
    let result = model_manager.generate(&prompt, Some(config)).await?;

    Ok(Json(GenerateResponse {
        text: result.text,
        model: result.model_used,
        usage: UsageInfo {
            prompt_tokens: prompt.len() / 4,
            completion_tokens: result.tokens_generated,
            total_tokens: prompt.len() / 4 + result.tokens_generated,
        },
        inference_time_ms: result.generation_time_secs * 1000.0,
    }))
}

/// 文本生成（流式）
pub async fn generate_stream(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Response {
    info!("Stream generate request: prompt length = {}", request.prompt.len());

    // 如果请求指定了模型，先切换
    if let Err(e) = ensure_model(&state, request.model.as_deref()).await {
        return axum::response::Json(serde_json::json!({
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

// ============ 聊天端点 ============

/// 聊天（非流式）
pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> AppResult<Json<ChatResponse>> {
    info!("Chat request: {} messages", request.messages.len());

    ensure_model(&state, request.model.as_deref()).await?;

    let model_manager = state.model_manager.lock().await;
    let prompt = model_manager.format_chat_prompt(&request.messages);
    
    let config = GenerationConfig {
        max_new_tokens: request.max_tokens.unwrap_or(256),
        temperature: request.temperature.unwrap_or(0.6),
        top_p: request.top_p.unwrap_or(0.9),
        seed: 42,
    };

    let result = model_manager.generate(&prompt, Some(config)).await?;

    Ok(Json(ChatResponse {
        message: ChatMessage {
            role: "assistant".to_string(),
            content: result.text,
        },
        model: result.model_used,
        usage: UsageInfo {
            prompt_tokens: prompt.len() / 4,
            completion_tokens: result.tokens_generated,
            total_tokens: prompt.len() / 4 + result.tokens_generated,
        },
        inference_time_ms: result.generation_time_secs * 1000.0,
    }))
}

/// 聊天（流式）
pub async fn chat_stream(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Response {
    info!("Stream chat request: {} messages", request.messages.len());

    if let Err(e) = ensure_model(&state, request.model.as_deref()).await {
        return axum::response::Json(serde_json::json!({
            "error": e.to_string()
        })).into_response();
    }

    let model_manager = state.model_manager.lock().await;
    let prompt = model_manager.format_chat_prompt(&request.messages);
    
    let config = GenerationConfig {
        max_new_tokens: request.max_tokens.unwrap_or(256),
        temperature: request.temperature.unwrap_or(0.6),
        top_p: request.top_p.unwrap_or(0.9),
        seed: 42,
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

// ============ OpenAI 兼容端点 ============

/// OpenAI 兼容 completions
pub async fn generate_completion(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if request.stream.unwrap_or(false) {
        return Err(AppError::InvalidRequest(
            "For streaming, use POST /generate/stream".to_string()
        ));
    }

    ensure_model(&state, request.model.as_deref()).await?;

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
    
    let result = model_manager.generate(&prompt, Some(config)).await?;

    Ok(Json(serde_json::json!({
        "id": format!("cmpl-{}", uuid_simple()),
        "object": "text_completion",
        "created": timestamp(),
        "model": result.model_used,
        "choices": [{
            "text": result.text,
            "index": 0,
            "logprobs": null,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": prompt.len() / 4,
            "completion_tokens": result.tokens_generated,
            "total_tokens": prompt.len() / 4 + result.tokens_generated
        }
    })))
}

/// OpenAI 兼容 chat completions
pub async fn chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if request.stream.unwrap_or(false) {
        return Err(AppError::InvalidRequest(
            "For streaming, use POST /chat/stream".to_string()
        ));
    }

    ensure_model(&state, request.model.as_deref()).await?;

    let model_manager = state.model_manager.lock().await;
    let prompt = model_manager.format_chat_prompt(&request.messages);
    
    let config = GenerationConfig {
        max_new_tokens: request.max_tokens.unwrap_or(256),
        temperature: request.temperature.unwrap_or(0.6),
        top_p: request.top_p.unwrap_or(0.9),
        seed: 42,
    };

    let result = model_manager.generate(&prompt, Some(config)).await?;

    Ok(Json(serde_json::json!({
        "id": format!("chatcmpl-{}", uuid_simple()),
        "object": "chat.completion",
        "created": timestamp(),
        "model": result.model_used,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": result.text
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": prompt.len() / 4,
            "completion_tokens": result.tokens_generated,
            "total_tokens": prompt.len() / 4 + result.tokens_generated
        }
    })))
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

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

fn timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
