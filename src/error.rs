use serde::{Serialize};

#[derive(Serialize)]
pub struct UnsupportedFileError {
    pub error: String,
    pub file_type: String,
}


#[derive(Serialize)]
pub struct RemoveFileError {
    pub error: String,
    pub file_id: String,
}

