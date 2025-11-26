use axum::{
    extract::{State},
    Json,
    Router,
    routing::{get, post},
    response::{sse::Event, Sse},
};
use serde::{Deserialize, Serialize};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use crate::AppState;


#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub is_healthy: bool,
    pub status: String,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateRequest {
    pub prompt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerationResponse {
    pub prompt: String,
}


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(healthy))
        .route("/generate", post(generate))
        .route("/generate/stream", post(streaming))
}


pub async fn healthy(State(_state): State<AppState>) -> Json<HealthResponse>{
    Json(HealthResponse{
        is_healthy : true,
        status: "OK".to_string(),
    })
}


pub async fn generate(State(state): State<AppState>, Json(request) : Json<GenerateRequest>) -> Json<GenerationResponse> {
    let model_manager = state.model_manager.lock().await;
    let result = model_manager
        .generate(&request.prompt)
        .await;

    Json(GenerationResponse {
        prompt: result.text,
    })
}


pub async fn streaming(State(state): State<AppState>, Json(request) : Json<GenerateRequest>)
    -> Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>> {
    let model_manager = state.model_manager.lock().await;

    let receiver = model_manager.stream(&request.prompt).await;

    let stream = ReceiverStream::new(receiver)
        .map(|chunk| {
            let data = serde_json::to_string(&chunk)
                .map_err(|e| axum::Error::new(e))?;
            Ok(Event::default().data(data))
        });

    Sse::new(stream)
    
}