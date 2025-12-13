use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct InferenceRequest {
    #[serde(rename = "model_name")]  //expected input format: model name:   , prompt: 
    pub model: String,
    pub prompt: String,
}

#[derive(Serialize)]
pub struct InferenceResponse {
    pub text: String,
}


#[derive(Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub filename: String,
    pub file_size: usize,
}