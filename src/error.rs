use serde::{Serialize};

#[derive(Serialize)]
pub struct UnsupportedFileError {
    pub error: String,
    pub file_type: String,
}