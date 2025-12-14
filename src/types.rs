use serde::{Serialize, Deserialize};
use crate::session::ChatMessage;

#[derive(Deserialize)]
pub struct InferenceRequest {
    #[serde(rename = "model_name")]  //expected input format: model name:   , prompt: 
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct InferenceResponse {
    pub text: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub session_id: Option<String>,
}


#[derive(Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub filename: String,
    pub file_size: usize,
}


#[derive(Serialize)]
pub struct DeleteResponse {
    pub file_id: String,
    pub result: bool,
}


#[derive(Serialize)]
pub struct RemoveSessionResponse {
    pub session_id: String,
    pub cleared: bool,
}


// 获取 session 的响应
#[derive(Serialize)]
pub struct GetSessionResponse {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
    pub exists: bool,
}


// 同步 session 的请求
#[derive(Deserialize)]
pub struct SyncSessionRequest {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
}


// 同步 session 的响应
#[derive(Serialize)]
pub struct SyncSessionResponse {
    pub session_id: String,
    pub synced: bool,
    pub message_count: usize,
}
