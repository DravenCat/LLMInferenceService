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
use reqwest::StatusCode;
use crate::AppState;
use crate::error::{RemoveFileError, RemoveSessionError, UnsupportedFileError};
use crate::file_parser::{parse_file, CacheFile};
use crate::types::{
    DeleteResponse, InferenceRequest, InferenceResponse, RemoveSessionResponse, UploadResponse
};
use crate::mistral_runner::{run_inference_collect, run_inference_stream};
use crate::session::{ChatMessage, SessionConfig, SessionHelper};

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

    Json(InferenceResponse {
        text,
        session_id: None,
    })
}

pub async fn infer_stream_handler(
    State(state): State<AppState>,
    Json(req): Json<InferenceRequest>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>>
{
    print!("infer_stream_handler entered!");
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(32);

    let model = req.model;
    let user_prompt = req.prompt;

    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let config = SessionConfig::default();

    let mut session = SessionHelper::get_or_create(
        &state.session_manager,
        &session_id,
        config
    ).await;

    let prompt = build_prompt(&state, &user_prompt).await;

    session.add_user_message(prompt);

    let messages: Vec<ChatMessage> = session.get_messages().to_vec();

    let session_manager = state.session_manager.clone();
    let session_id_clone = session_id.clone();

    tokio::spawn(async move {
        let mut full_response = String::new();

        if let Ok(mut stream) = run_inference_stream(&model, &messages).await {
            while let Some(token) = stream.next().await {
                full_response.push_str(&token);
                if tx.send(token).await.is_err() {
                    break;
                }
            }
        }

        if !full_response.is_empty() {
            let mut session = SessionHelper::get_or_create(
                &session_manager,
                &session_id_clone,
                SessionConfig::default(),
            ).await;
            session.add_assistant_message(full_response);
            SessionHelper::update(&session_manager, session).await;
        }

        // 发送会话 ID（作为特殊消息）
        let session_info = serde_json::json!({
            "session_id": session_id_clone,
            "type": "session_info"
        }).to_string();
        let _ = tx.send(format!("__SESSION__:{}", session_info)).await;

        let _ = tx.send("[DONE]".to_string()).await;
    });

    let sse_stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|token| {
            if token == "[DONE]" {
                return Ok(Event::default().data("[DONE]"));
            }

            if token.starts_with("__SESSION__:") {
                let session_data = &token["__SESSION__:".len()..];
                return Ok(Event::default().event("session").data(session_data));
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


async fn build_prompt(state: &AppState, prompt: &String) -> String {
    let mut cache = state.file_cache.write().await;
    let mut final_prompt = String::new();
    for (_, value) in cache.iter() {
        final_prompt.push_str(
            format!("Content in {} : {}\n", value.filename, value.content)
                .as_str());
    }
    cache.clear();

    final_prompt.push_str(
        format!("User's prompt: {}\n", prompt)
            .as_str()
    );

    final_prompt
}


pub async fn upload_handler(
    State(state): State<AppState>,
    mut multipart : Multipart)
    -> Result<Json<UploadResponse>, (StatusCode, Json<UnsupportedFileError>)> {
    let item = multipart.next_field().await.unwrap().unwrap();
    let filename = item
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());

    let extension = Path::new(&filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let allowed_extension = vec!["txt", "pdf", "docx", "pptx", "xlsx"];
    if !allowed_extension.contains(&extension.to_lowercase().as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UnsupportedFileError {
                error : "Unsupported file type".to_string(),
                file_type : extension.to_string()
            })
        ))
    }

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
    Ok(Json(UploadResponse {
        file_id,
        filename,
        file_size
    }))
}


pub async fn remove_handler(State(state): State<AppState>,
                            axum::extract::Path(file_id): axum::extract::Path<String>)
    -> Result<Json<DeleteResponse>, (StatusCode, Json<RemoveFileError>)> {
    let mut cache = state.file_cache.write().await;
    match cache.get(&file_id) {
        Some(_) => {
            cache.remove(&file_id);
        }
        None => {
            return Err((StatusCode::BAD_REQUEST,
                Json(RemoveFileError {
                error : "File does not exist".to_string(),
                file_id : file_id.to_string()
            })))
        }
    }
    println!("Current number of files in cache: {}", cache.len());

    let delete_response = DeleteResponse {
        file_id,
        result: true,
    };

    Ok(Json(delete_response))
}


pub async fn remove_session_handler(State(state): State<AppState>,
                                    axum::extract::Path(session_id): axum::extract::Path<String>)
    -> Result<Json<RemoveSessionResponse>, (StatusCode, Json<RemoveSessionError>)> {
    if !SessionHelper::remove(&state.session_manager, &session_id).await {
        return Err(
            (StatusCode::BAD_REQUEST,
            Json(RemoveSessionError {
                error : "Session does not exist".to_string(),
                session_id : session_id.to_string()
            }))
        )
    }

    Ok(Json(RemoveSessionResponse {
        session_id,
        cleared: true
    }))
}


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(infer_handler))
        .route("/generate/stream", post(infer_stream_handler))
        .route("/health", get(healthy))
        .route("/upload", post(upload_handler))
        .route("/files/{file_id}", delete(remove_handler))
        .route("/sessions/{session_id}", delete(remove_session_handler))
}