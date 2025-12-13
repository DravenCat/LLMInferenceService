use serde::{Serialize};

#[derive(Serialize)]
pub struct UnsupportedFileError {
    pub error: String,
    pub file_type: String,
}


#[derive(Serialize)]
pub struct RemoveFileError {
    pub error: String,
    pub file_name: String,
}


#[derive(Serialize)]
pub struct UploadFileError {
    pub error: String,
    pub file_name: String,
}