use axum::{
    extract::{State},
    Json,
    Router,
    routing::{get, post},
    response::{sse::Event, Sse},
};
use serde::{Deserialize, Serialize};
use tokio_stream::{StreamExt};
use std::{time::Duration};
use crate::AppState;


#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub is_healthy: bool,
    pub status: String,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub model_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerationResponse {
    content: String,
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
        content: result.text,
    })
}


pub async fn streaming(State(state): State<AppState>, Json(request) : Json<GenerateRequest>)
    -> Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>> {
    let _model_manager = state.model_manager.lock().await;
    let model_name = request.model_name.clone();

    // get actual content from the model manager
    // let result = model_manager.stream(&request.prompt).await;
    let result = format!("You are using {model_name}. This is the test message");
    let chars: Vec<String> = result.chars().map(|c| c.to_string()).collect();

    let stream = tokio_stream::iter(chars.into_iter().map(|content| {
        let response = GenerationResponse { content};
        let json = serde_json::to_string(&response).unwrap();
        Ok(Event::default().data(json))
    }))
        .then(|event| async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            event
        })
        .chain(tokio_stream::once(Ok(Event::default().data("[DONE]"))));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}