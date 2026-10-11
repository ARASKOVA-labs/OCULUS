use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OcrBlock {
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OcrPage {
    pub page: u32,
    pub text: String,
    pub blocks: Vec<OcrBlock>,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OcrResult {
    pub document_id: String,
    pub filename: String,
    pub language: String,
    pub pages: Vec<OcrPage>,
    pub processing_time_ms: u64,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PreprocessOptions {
    pub grayscale: bool,
    pub denoise: bool,
    pub deskew: bool,
    pub threshold: bool,
    pub orientation_detect: bool,
    pub contrast_norm: bool,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            grayscale: true,
            denoise: false,
            deskew: false,
            threshold: false,
            orientation_detect: false,
            contrast_norm: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentDescriptor {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub kind: String,
    pub size_bytes: u64,
    pub is_valid: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobProgress {
    pub job_id: String,
    pub document_id: String,
    pub state: String,
    pub progress_percent: u32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportRequest {
    pub document_id: String,
    pub format: String, // "txt", "md", "json", "pdf"
    pub result: OcrResult,
    pub destination_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportResponse {
    pub success: bool,
    pub destination_path: String,
    pub bytes_written: usize,
    pub error: Option<String>,
}
