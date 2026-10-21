use crate::ffi::NativeEngine;
use crate::models::{OcrResult, PreprocessOptions};
use crate::pdf::PdfPipeline;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchItem {
    pub id: String,
    pub path: String,
    pub language: String,
    pub options: PreprocessOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchProgressEvent {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file: String,
    pub progress_percent: u32,
    pub current_stage: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchFileResult {
    pub id: String,
    pub filename: String,
    pub success: bool,
    pub result: Option<OcrResult>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchSummary {
    pub total_files: usize,
    pub succeeded_files: usize,
    pub failed_files: usize,
    pub total_pages: usize,
    pub total_duration_ms: u64,
    pub results: Vec<BatchFileResult>,
}

pub struct BatchEngine;

impl BatchEngine {
    /// Execute a concurrent batch job with bounded worker pool,
    /// real-time progress, responsive cancellation, and per-file error isolation.
    pub async fn run_batch<F>(
        engine: Arc<NativeEngine>,
        items: Vec<BatchItem>,
        max_concurrency: usize,
        cancel_flag: Arc<AtomicBool>,
        progress_sender: F,
    ) -> Result<BatchSummary, String>
    where
        F: Fn(BatchProgressEvent) + Send + Sync + 'static,
    {
        if items.is_empty() {
            return Ok(BatchSummary {
                total_files: 0,
                succeeded_files: 0,
                failed_files: 0,
                total_pages: 0,
                total_duration_ms: 0,
                results: Vec::new(),
            });
        }

        let start_time = Instant::now();
        let total_files = items.len();
        let concurrency = if max_concurrency > 0 {
            max_concurrency
        } else {
            4
        };
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let progress_fn = Arc::new(progress_sender);

        let mut tasks = Vec::with_capacity(total_files);

        for (idx, item) in items.into_iter().enumerate() {
            let sem = semaphore.clone();
            let eng = engine.clone();
            let cancel = cancel_flag.clone();
            let prog = progress_fn.clone();

            tasks.push(tokio::spawn(async move {
                let _permit = match sem.acquire().await {
                    Ok(p) => p,
                    Err(_) => {
                        return BatchFileResult {
                            id: item.id.clone(),
                            filename: "semaphore_error".to_string(),
                            success: false,
                            result: None,
                            error: Some("Failed to acquire concurrency permit".to_string()),
                        }
                    }
                };

                let filename = Path::new(&item.path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "document".to_string());

                if cancel.load(Ordering::SeqCst) {
                    return BatchFileResult {
                        id: item.id,
                        filename,
                        success: false,
                        result: None,
                        error: Some("Job cancelled by user".to_string()),
                    };
                }

                // Read file
                let bytes = match crate::commands::guard::readable_file(&item.path)
                    .and_then(|p| fs::read(p).map_err(|e| e.to_string()))
                {
                    Ok(b) => b,
                    Err(e) => {
                        return BatchFileResult {
                            id: item.id,
                            filename,
                            success: false,
                            result: None,
                            error: Some(format!("Failed to read file: {}", e)),
                        }
                    }
                };

                // Determine if PDF or image
                let ocr_res = if bytes.starts_with(b"%PDF") {
                    PdfPipeline::process_pdf(
                        &eng,
                        &item.id,
                        &filename,
                        &bytes,
                        &item.language,
                        item.options,
                        &cancel,
                        |_curr, _tot, _msg| {},
                    )
                } else {
                    eng.run_ocr(&item.id, &filename, &bytes, &item.language, item.options)
                };

                let percent = (((idx + 1) as f32 / total_files as f32) * 100.0) as u32;
                prog(BatchProgressEvent {
                    total_files,
                    completed_files: idx + 1,
                    current_file: filename.clone(),
                    progress_percent: percent,
                    current_stage: if ocr_res.is_ok() {
                        "Completed".to_string()
                    } else {
                        "Failed".to_string()
                    },
                });

                match ocr_res {
                    Ok(res) => BatchFileResult {
                        id: item.id,
                        filename,
                        success: true,
                        result: Some(res),
                        error: None,
                    },
                    Err(err) => BatchFileResult {
                        id: item.id,
                        filename,
                        success: false,
                        result: None,
                        error: Some(err),
                    },
                }
            }));
        }

        let mut results = Vec::with_capacity(tasks.len());
        let mut succeeded_files = 0;
        let mut failed_files = 0;
        let mut total_pages = 0;

        for task in tasks {
            match task.await {
                Ok(file_res) => {
                    if file_res.success {
                        succeeded_files += 1;
                        if let Some(ref r) = file_res.result {
                            total_pages += r.pages.len();
                        }
                    } else {
                        failed_files += 1;
                    }
                    results.push(file_res);
                }
                Err(e) => {
                    failed_files += 1;
                    results.push(BatchFileResult {
                        id: "panic".to_string(),
                        filename: "worker_panic".to_string(),
                        success: false,
                        result: None,
                        error: Some(format!("Worker panicked: {}", e)),
                    });
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(BatchSummary {
            total_files,
            succeeded_files,
            failed_files,
            total_pages,
            total_duration_ms: elapsed_ms,
            results,
        })
    }
}
