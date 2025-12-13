use axum::{
    extract::{State, Multipart},
    Json,
    Router,
    routing::{get, post},
    response::{sse::Event, Sse},
};
use serde::{Deserialize, Serialize};
use tokio_stream::{StreamExt};
use std::{time::Duration};
use std::path::Path;
use axum::routing::delete;
use crate::AppState;
use crate::file_parser::{parse_file, CacheFile};
use crate::types::{DeleteResponse, InferenceRequest, InferenceResponse, UploadResponse};
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


pub async fn upload_handler(State(state): State<AppState>, mut multipart : Multipart) -> Json<UploadResponse> {
    let item = multipart.next_field().await.unwrap().unwrap();
    let filename = item.file_name().map(|s| s.to_string()).unwrap_or_else(|| "".to_string());

    let data = item.bytes().await.unwrap();
    let file_size = data.len();

    let content = parse_file(Path::new(&filename), &data).await.unwrap();
    let file_id = uuid::Uuid::new_v4().to_string();
    {
        println!("file_id: {}, file_content: {}", file_id, content);
    }
    let cache_file = CacheFile {
        filename: filename.clone(),
        content,
    };
    {
        let mut cache = state.file_cache.write().await;
        cache.insert(file_id.clone(), cache_file);
        println!("Current number of files in cache: {}", cache.len());
    }
    Json(UploadResponse {
        file_id,
        filename,
        file_size
    })
}


pub async fn remove_handler(State(state): State<AppState>,
                            axum::extract::Path(file_id): axum::extract::Path<String>)
    -> Json<DeleteResponse> {
    let mut cache = state.file_cache.write().await;
    cache.remove(&file_id);
    println!("Current number of files in cache: {}", cache.len());

    let delete_response = DeleteResponse {
        file_id,
        result: true,
    };

    Json(delete_response)
}


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(infer_handler))
        .route("/generate/stream", post(infer_stream_handler))
        .route("/health", get(healthy))
        .route("/upload", post(upload_handler))
        .route("/files/{file_id}", delete(remove_handler))
}