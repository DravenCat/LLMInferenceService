use serde::{Serialize, Deserialize};

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