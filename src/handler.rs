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
use tower_http::follow_redirect::policy::PolicyExt;
use crate::AppState;
use crate::error::{RemoveFileError, RemoveSessionError, UnsupportedFileError};
use crate::file_parser::{parse_file, CacheFile};
use crate::types::{
    DeleteResponse, InferenceRequest, InferenceResponse, RemoveSessionResponse, UploadResponse,
    GetSessionResponse, SyncSessionRequest, SyncSessionResponse
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
    println!("infer_stream_handler entered!");
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

    // 如果有文件，先添加文件内容作为单独的 user message
    if let Some(file_context) = build_file_context(&state).await {
        println!("Adding file context to session: {} bytes", file_context.len());
        session.add_user_message(file_context);
    }
    
    // 添加用户的实际 prompt
    session.add_user_message(user_prompt);

    // 保存 session（包含文件内容和用户消息）
    SessionHelper::update(&state.session_manager, session.clone()).await;

    let messages: Vec<ChatMessage> = session.get_messages().to_vec();
    
    println!("Total messages in session: {}", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        println!("  Message {}: role={:?}, content_len={}", i, msg.role, msg.content.len());
    }

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


/// 构建文件内容的 prompt（如果有文件的话）
async fn build_file_context(state: &AppState) -> Option<String> {
    let mut cache = state.file_cache.write().await;
    
    println!("build_file_context: cache size = {}", cache.len());
    
    if cache.is_empty() {
        println!("build_file_context: no files in cache");
        return None;
    }
    
    let mut file_context = String::from("I'm sharing the following file(s) with you:\n\n");
    
    for (_, value) in cache.iter() {
        println!("build_file_context: processing file {} ({}), content_len={}", 
            value.filename, value.extension, value.content.len());
        match value.extension.as_str() {
            "txt" => {
                file_context.push_str(
                    format!("=== Text File: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "md" => {
                file_context.push_str(
                    format!("=== Markdown File: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "pdf" => {
                file_context.push_str(
                    format!("=== PDF File: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "docx" => {
                file_context.push_str(
                    format!("=== Word Document: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "pptx" => {
                file_context.push_str(
                    format!("=== PowerPoint: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "xlsx" => {
                file_context.push_str(
                    format!("=== Excel Spreadsheet: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
            "py" | "js" | "ts" | "jsx" | "tsx" | "vue" | "svelte" |
            "rs" | "go" | "java" | "kt" | "scala" |
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" |
            "cs" | "fs" | "rb" | "php" | "pl" | "pm" |
            "swift" | "m" | "mm" | "r" | "R" | "jl" |
            "lua" | "tcl" | "awk" | "sed" |
            "hs" | "ml" | "elm" | "clj" | "cljs" | "ex" | "exs" |
            "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" |
            "sql" | "prisma" | "graphql" | "gql" |
            "html" | "htm" | "css" | "scss" | "sass" | "less" |
            "xml" | "xsl" | "xslt" |
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" |
            "log" | "env" | "makefile" | "cmake" | "dockerfile" |
            "gitignore" | "editorconfig"
            => {
                file_context.push_str(
                    format!("=== {} Code File: {} ===\n{}\n\n", 
                        value.extension.to_uppercase(), value.filename, value.content)
                        .as_str());
            }
            _ => {
                file_context.push_str(
                    format!("=== File: {} ===\n{}\n\n", value.filename, value.content)
                        .as_str());
            }
        }
    }
    
    file_context.push_str("Please refer to the above file content(s) when answering my questions.");
    
    cache.clear();
    
    Some(file_context)
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

    let allowed_text_file = vec!["txt", "pdf", "docx", "pptx", "xlsx", "md"];
    let allowed_code_file = vec![
            "py", "js", "ts", "jsx", "tsx", "vue", "svelte",      // Web
            "rs",                                                 // Rust
            "go",                                                 // go
            "java", "kt", "scala",                                // java
            "c", "cpp", "cc", "cxx", "h", "hpp", "hxx",           // C/C++
            "cs", "fs",                                           // .NET
            "rb", "php", "pl", "pm",                              // php
            "swift", "m", "mm",                                   // Apple
            "r", "R", "jl",                                       // data science
            "lua", "tcl", "awk", "sed",                           // Script
            "hs", "ml", "elm", "clj", "cljs", "ex", "exs",        // function
            "sh", "bash", "zsh", "fish", "bat", "cmd", "ps1",     // Shell
            "sql", "prisma", "graphql", "gql",                    // database
            "html", "htm", "css", "scss", "sass", "less",         // Web page
            "xml", "xsl", "xslt",                                 // XML
            "json", "yaml", "yml", "toml", "ini", "cfg", "conf",  // config
            "log", "env",                                         // log
            "makefile", "cmake", "dockerfile",                    // build
            "gitignore", "editorconfig"                           // git
    ];
    if !allowed_text_file.contains(&extension.to_lowercase().as_str())
    && !allowed_code_file.contains(&extension.to_lowercase().as_str()){
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
        extension : extension.to_string(),
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


/// 获取 session 信息
pub async fn get_session_handler(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>
) -> Json<GetSessionResponse> {
    match SessionHelper::get(&state.session_manager, &session_id).await {
        Some(session) => {
            Json(GetSessionResponse {
                session_id,
                messages: session.messages,
                exists: true,
            })
        }
        None => {
            Json(GetSessionResponse {
                session_id,
                messages: vec![],
                exists: false,
            })
        }
    }
}


/// 同步 session 消息（前端切换 session 时调用）
pub async fn sync_session_handler(
    State(state): State<AppState>,
    Json(req): Json<SyncSessionRequest>
) -> Json<SyncSessionResponse> {
    let config = SessionConfig::default();
    
    // 过滤掉前端消息中可能存在的文件信息，只保留 role 和 content
    let messages: Vec<ChatMessage> = req.messages.into_iter().map(|msg| {
        ChatMessage {
            role: msg.role,
            content: msg.content,
        }
    }).collect();
    
    let message_count = messages.len();
    
    let session_manager = state.session_manager.read().await;
    let session = session_manager.get(req.session_id.as_str()).unwrap();
    
    println!("Session {} synced with {} messages", req.session_id, session.messages.len());
    
    Json(SyncSessionResponse {
        session_id: req.session_id,
        synced: true,
        message_count,
    })
}


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(infer_handler))
        .route("/generate/stream", post(infer_stream_handler))
        .route("/health", get(healthy))
        .route("/upload", post(upload_handler))
        .route("/files/{file_id}", delete(remove_handler))
        .route("/sessions/{session_id}", delete(remove_session_handler))
        .route("/sessions/{session_id}", get(get_session_handler))
        .route("/sessions/sync", post(sync_session_handler))
}