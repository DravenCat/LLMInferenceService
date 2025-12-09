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

use crate::types::{InferenceRequest, InferenceResponse};
use crate::mistral_runner::{run_inference_collect, run_inference_stream};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub is_healthy: bool,
    pub status: String,
}


pub async fn healthy(State(_state): State<AppState>) -> Json<HealthResponse>{
    Json(HealthResponse{
        is_healthy : true,
        status: "OK".to_string(),
    })
}

//modified to join the inferrence part
pub async fn infer_handler(
    Json(req): Json<InferenceRequest>,
) -> Json<InferenceResponse> {
    let text = run_inference_collect(req.model.as_str(), req.prompt.as_str())
        .await
        .unwrap_or_else(|_| "Inference failed".to_string());

    Json(InferenceResponse { text })
}

pub async fn infer_stream_handler(
    Json(req): Json<InferenceRequest>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>>
{
    print!("infer_stream_handler entered!");
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(32);

    let model = req.model;
    let prompt = req.prompt;

    tokio::spawn(async move {
        if let Ok(mut stream) = run_inference_stream(model.as_str(), prompt.as_str()).await {
            while let Some(token) = stream.next().await {
                if tx.send(token).await.is_err() {
                    break;
                }
            }
        }
        let _ = tx.send("[DONE]".to_string()).await;
    });

    let sse_stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|token| {
            if token == "[DONE]" {
                return Ok(Event::default().data("[DONE]"));
            }

            let json = serde_json::json!({
            "content": token
        })
                .to_string();

            Ok(Event::default().data(json))
        });

    println!("1111");

    Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )

}
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(infer_handler))
        .route("/generate/stream", post(infer_stream_handler))
        .route("/health", get(healthy))
}