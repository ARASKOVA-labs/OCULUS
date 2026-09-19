use crate::batch::{BatchEngine, BatchItem, BatchProgressEvent, BatchSummary};
use crate::ffi::NativeEngine;
use crate::intelligence::{DocumentIntelligence, StructuredDocument};
use crate::models::{
    DocumentDescriptor, ExportRequest, ExportResponse, JobProgress, OcrResult, PreprocessOptions,
};
use crate::pdf::PdfPipeline;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub struct AppState {
    pub engine: Arc<NativeEngine>,
    pub cancel_flag: Arc<AtomicBool>,
}

const MAX_FILE_SIZE_BYTES: u64 = 250 * 1024 * 1024; // 250 MB

#[tauri::command]
pub async fn inspect_document(path: String) -> Result<DocumentDescriptor, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let canonical = file_path
        .canonicalize()
        .map_err(|e| format!("Invalid file path: {}", e))?;

    let metadata = fs::metadata(&canonical).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let size_bytes = metadata.len();

    if size_bytes > MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "File size exceeds safety limit of 250MB (size: {} MB)",
            size_bytes / (1024 * 1024)
        ));
    }

    let filename = canonical
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    let ext = canonical
        .extension()
        .map(|e| e.to_string_lossy().to_uppercase())
        .unwrap_or_else(|| "FILE".to_string());

    // Read header magic bytes
    let file_bytes = fs::read(&canonical).map_err(|e| format!("Failed to read file: {}", e))?;
    let header_len = file_bytes.len().min(1024);
    let header = &file_bytes[..header_len];
    let is_pdf = header.windows(4).any(|w| w == b"%PDF");

    let (is_valid, width, height, err_msg) = if file_bytes.len() >= 4 {
        if file_bytes.starts_with(b"\x89PNG") || file_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            match image::load_from_memory(&file_bytes) {
                Ok(img) => (true, Some(img.width()), Some(img.height()), None),
                Err(e) => (false, None, None, Some(format!("Image decode error: {}", e))),
            }
        } else if is_pdf {
            let pages = NativeEngine::pdf_page_count(&file_bytes).unwrap_or(1);
            (true, Some(pages), None, None)
        } else if file_bytes.starts_with(b"II*\0") || file_bytes.starts_with(b"MM\0*") {
            // TIFF document
            (true, None, None, None)
        } else {
            (false, None, None, Some("Unsupported file format".to_string()))
        }
    } else {
        (false, None, None, Some("File too small to be valid".to_string()))
    };

    Ok(DocumentDescriptor {
        id: uuid::Uuid::new_v4().to_string(),
        path: canonical.to_string_lossy().into_owned(),
        filename,
        kind: ext,
        size_bytes,
        is_valid,
        width,
        height,
        error: err_msg,
    })
}

#[tauri::command]
pub async fn preprocess_image(
    path: String,
    options: PreprocessOptions,
) -> Result<String, String> {
    let canonical = Path::new(&path)
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    let bytes = fs::read(&canonical).map_err(|e| format!("Failed to read image for preprocessing: {}", e))?;
    let preprocessed = NativeEngine::preprocess_image(&bytes, options)?;

    let temp_path = std::env::temp_dir().join(format!("oculus_prep_{}.png", uuid::Uuid::new_v4()));
    fs::write(&temp_path, preprocessed).map_err(|e| format!("Failed to write preprocessed image: {}", e))?;

    Ok(temp_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn run_ocr(
    app: AppHandle,
    state: State<'_, AppState>,
    document_id: String,
    path: String,
    language: String,
    options: PreprocessOptions,
    file_bytes: Option<Vec<u8>>,
) -> Result<OcrResult, String> {
    state.cancel_flag.store(false, Ordering::SeqCst);

    let (filename, fname, bytes) = if Path::new(&path).is_file() {
        let canonical = Path::new(&path)
            .canonicalize()
            .map_err(|e| format!("Invalid path '{}': {}", path, e))?;
        let f = canonical
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "document".to_string());
        let full_str = canonical.to_string_lossy().into_owned();
        let b = fs::read(&canonical)
            .map_err(|e| format!("Failed to read document {}: {}", path, e))?;
        (f, full_str, b)
    } else {
        match file_bytes {
            Some(b) if !b.is_empty() => {
                let f = Path::new(&path)
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| {
                        if path.is_empty() {
                            "document".to_string()
                        } else {
                            path.clone()
                        }
                    });
                (f.clone(), f, b)
            }
            _ => {
                return Err(format!("Document file not found: '{}'", path));
            }
        }
    };

    // Emit job started
    let _ = app.emit(
        "job_progress",
        JobProgress {
            job_id: document_id.clone(),
            document_id: document_id.clone(),
            state: "Reading file".to_string(),
            progress_percent: 10,
            message: format!("Ingesting {}", filename),
        },
    );

    if bytes.len() as u64 > MAX_FILE_SIZE_BYTES {
        return Err("Document exceeds safety limit of 250MB".to_string());
    }

    if bytes.is_empty() {
        return Err("Cannot process empty document (0 bytes)".to_string());
    }

    if bytes.len() < 4 {
        return Err("Document file is too small to be a valid image or PDF".to_string());
    }

    // Validate magic bytes (PDF standard allows %PDF within the first 1024 bytes)
    let header_len = bytes.len().min(1024);
    let header = &bytes[..header_len];
    let is_pdf = header.windows(4).any(|w| w == b"%PDF");
    let is_png = bytes.starts_with(b"\x89PNG");
    let is_jpg = bytes.starts_with(&[0xFF, 0xD8, 0xFF]);
    let is_tiff = bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*");

    if !is_pdf && !is_png && !is_jpg && !is_tiff {
        return Err(format!(
            "Unsupported file format for '{}'. Expected PDF, PNG, JPEG, or TIFF.",
            filename
        ));
    }

    if state.cancel_flag.load(Ordering::SeqCst) {
        return Err("Job cancelled by user".to_string());
    }

    // Cache bytes for document_id in temp directory so subsequent exports always have access to source bytes
    let cache_dir = std::env::temp_dir().join("oculus_cache");
    let _ = fs::create_dir_all(&cache_dir);
    let cache_file = cache_dir.join(&document_id);
    let _ = fs::write(&cache_file, &bytes);

    let engine = state.engine.clone();
    let doc_id = document_id.clone();
    let lang = language.clone();
    let cancel = state.cancel_flag.clone();
    let app_handle = app.clone();

    // Route PDF vs Image
    let res = if is_pdf {
        tokio::task::spawn_blocking(move || {
            PdfPipeline::process_pdf(
                &engine,
                &doc_id,
                &fname,
                &bytes,
                &lang,
                options,
                &cancel,
                |curr, total, stage| {
                    let percent = (((curr as f32) / (total as f32)) * 80.0) as u32 + 10;
                    let _ = app_handle.emit(
                        "job_progress",
                        JobProgress {
                            job_id: doc_id.clone(),
                            document_id: doc_id.clone(),
                            state: format!("Page {}/{}", curr, total),
                            progress_percent: percent,
                            message: stage.to_string(),
                        },
                    );
                },
            )
        })
        .await
        .map_err(|e| format!("Worker thread panicked: {}", e))??
    } else {
        let _ = app.emit(
            "job_progress",
            JobProgress {
                job_id: document_id.clone(),
                document_id: document_id.clone(),
                state: "Preprocessing".to_string(),
                progress_percent: 30,
                message: "Applying image filters".to_string(),
            },
        );

        tokio::task::spawn_blocking(move || {
            engine.run_ocr(&doc_id, &fname, &bytes, &lang, options)
        })
        .await
        .map_err(|e| format!("Worker thread panicked: {}", e))??
    };

    let total_chars: usize = res.pages.iter().map(|p| p.text.len()).sum();
    let _ = app.emit(
        "job_progress",
        JobProgress {
            job_id: document_id,
            document_id: res.document_id.clone(),
            state: "Completed".to_string(),
            progress_percent: 100,
            message: format!(
                "Processed {} page(s), extracted {} characters in {}ms",
                res.pages.len(),
                total_chars,
                res.processing_time_ms
            ),
        },
    );

    Ok(res)
}

#[tauri::command]
pub async fn cancel_ocr(state: State<'_, AppState>) -> Result<bool, String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(true)
}

#[tauri::command]
pub async fn start_batch_ocr(
    app: AppHandle,
    state: State<'_, AppState>,
    items: Vec<BatchItem>,
    max_concurrency: Option<usize>,
) -> Result<BatchSummary, String> {
    state.cancel_flag.store(false, Ordering::SeqCst);
    let app_handle = app.clone();

    BatchEngine::run_batch(
        state.engine.clone(),
        items,
        max_concurrency.unwrap_or(4),
        state.cancel_flag.clone(),
        move |event: BatchProgressEvent| {
            let _ = app_handle.emit("batch_progress", event);
        },
    )
    .await
}

#[tauri::command]
pub async fn analyze_document(result: OcrResult) -> Result<StructuredDocument, String> {
    Ok(DocumentIntelligence::analyze(&result))
}

fn resolve_default_export_dir(source_filename: &str) -> PathBuf {
    // 1. If source_filename is an absolute path and its parent exists and is not root, use its parent
    let source_p = Path::new(source_filename);
    if source_p.is_absolute() {
        if let Some(parent) = source_p.parent() {
            if parent.exists() && parent != Path::new("/") {
                return parent.to_path_buf();
            }
        }
    }

    // 2. Try ~/Downloads or ~/Documents
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let downloads = PathBuf::from(&home).join("Downloads");
        if downloads.exists() {
            return downloads;
        }
        let documents = PathBuf::from(&home).join("Documents");
        if documents.exists() {
            return documents;
        }
        let home_p = PathBuf::from(&home);
        if home_p.exists() && home_p != Path::new("/") {
            return home_p;
        }
    }

    // 3. Fallback to temp directory
    let temp = std::env::temp_dir();
    if temp.exists() && temp != Path::new("/") {
        return temp;
    }

    PathBuf::from(".")
}

pub fn perform_export(
    engine: &NativeEngine,
    req: ExportRequest,
) -> Result<ExportResponse, String> {
    let filename_stem = Path::new(&req.result.filename)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());

    let target_path = match req.destination_path {
        Some(p) => {
            let pb = PathBuf::from(&p);
            if pb.parent() == Some(Path::new("/")) || pb == Path::new("/") {
                let fname = pb
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| {
                        format!("{}_extracted.{}", filename_stem, req.format.to_lowercase())
                    });
                let base_dir = resolve_default_export_dir(&req.result.filename);
                base_dir.join(fname)
            } else {
                pb
            }
        }
        None => {
            let ext = req.format.to_lowercase();
            let base_dir = resolve_default_export_dir(&req.result.filename);
            base_dir.join(format!("{}_extracted.{}", filename_stem, ext))
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() && parent != Path::new("/") {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create export directory '{}': {}", parent.display(), e))?;
        }
    }

    // If searchable PDF export is requested
    if req.format.to_lowercase() == "pdf" {
        let output_base = target_path
            .with_extension("")
            .to_string_lossy()
            .into_owned();

        let source_path = Path::new(&req.result.filename);
        let cache_file = std::env::temp_dir().join("oculus_cache").join(&req.result.document_id);

        let source_bytes = if source_path.exists() && !req.result.filename.is_empty() {
            fs::read(source_path)
                .map_err(|e| format!("Failed to read source file '{}': {}", req.result.filename, e))?
        } else if cache_file.exists() {
            fs::read(&cache_file)
                .map_err(|e| format!("Failed to read cached document bytes: {}", e))?
        } else {
            return Err(format!(
                "Source document file not found at '{}' for PDF generation",
                req.result.filename
            ));
        };

        // Determine if source is a PDF or an image
        let sample_png = if source_bytes.starts_with(b"%PDF") {
            crate::ffi::NativeEngine::render_pdf_page(&source_bytes, 1, 300)
                .map_err(|e| format!("Render page for PDF export error: {}", e))?
        } else {
            // Source is already an image (PNG, JPEG, TIFF). Pass directly to engine!
            source_bytes
        };

        engine.create_searchable_pdf(&sample_png, &req.result.language, &output_base)?;

        let final_pdf = PathBuf::from(format!("{}.pdf", output_base));
        let size = fs::metadata(&final_pdf).map(|m| m.len() as usize).unwrap_or(0);

        return Ok(ExportResponse {
            success: true,
            destination_path: final_pdf.to_string_lossy().into_owned(),
            bytes_written: size,
            error: None,
        });
    }

    let output_content = match req.format.to_lowercase().as_str() {
        "txt" => req
            .result
            .pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n--- Page Break ---\n\n"),
        "md" => {
            let mut md = format!("# OCR Extraction: {}\n\n", req.result.filename);
            md.push_str(&format!("- **Language**: `{}`\n", req.result.language));
            md.push_str(&format!(
                "- **Processing time**: `{}ms`\n",
                req.result.processing_time_ms
            ));
            md.push_str(&format!("- **Total Pages**: `{}`\n\n", req.result.pages.len()));
            for page in &req.result.pages {
                md.push_str(&format!(
                    "## Page {}\n*Confidence: {:.1}%*\n\n",
                    page.page, page.confidence
                ));
                md.push_str(&page.text);
                md.push_str("\n\n");
            }
            md
        }
        "json" => serde_json::to_string_pretty(&req.result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?,
        "jsonl" => {
            let mut lines = Vec::new();
            for page in &req.result.pages {
                let obj = serde_json::json!({
                    "document": req.result.filename,
                    "document_id": req.result.document_id,
                    "page": page.page,
                    "confidence": page.confidence,
                    "language": req.result.language,
                    "text": page.text,
                    "quality": if page.confidence > 80.0 { "high" } else if page.confidence > 50.0 { "medium" } else { "low" },
                });
                lines.push(
                    serde_json::to_string(&obj).map_err(|e| format!("JSON line error: {}", e))?,
                );
            }
            lines.join("\n")
        }
        _ => return Err(format!("Unsupported export format: {}", req.format)),
    };

    let mut file = fs::File::create(&target_path)
        .map_err(|e| format!("Failed to create destination file {}: {}", target_path.display(), e))?;

    file.write_all(output_content.as_bytes())
        .map_err(|e| format!("Failed to write export content: {}", e))?;

    Ok(ExportResponse {
        success: true,
        destination_path: target_path.to_string_lossy().into_owned(),
        bytes_written: output_content.len(),
        error: None,
    })
}

#[tauri::command]
pub async fn export_result(
    state: State<'_, AppState>,
    req: ExportRequest,
) -> Result<ExportResponse, String> {
    perform_export(&state.engine, req)
}

#[tauri::command]
pub async fn render_pdf_page(
    file_bytes: Vec<u8>,
    page_number: u32,
    dpi: u32,
) -> Result<Vec<u8>, String> {
    if !file_bytes.starts_with(b"%PDF") {
        return Err("File is not a valid PDF document".to_string());
    }
    crate::ffi::NativeEngine::render_pdf_page(&file_bytes, page_number, dpi)
}

#[tauri::command]
pub async fn read_binary_file(path: String) -> Result<Vec<u8>, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    fs::read(p).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to show in folder: {}", e))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Ok(())
    }
}

#[tauri::command]
pub async fn repair_pdf_native(file_bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if file_bytes.is_empty() {
        return Err("Cannot repair empty PDF file".to_string());
    }
    let mut doc = lopdf::Document::load_mem(&file_bytes)
        .map_err(|e| format!("PDF corrupted beyond recovery: {}", e))?;
    doc.prune_objects();
    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| format!("Failed to reconstruct PDF: {}", e))?;
    Ok(out)
}

#[tauri::command]
pub async fn protect_pdf_native(file_bytes: Vec<u8>, password: String) -> Result<Vec<u8>, String> {
    if file_bytes.is_empty() {
        return Err("Cannot protect empty PDF file".to_string());
    }
    if password.trim().is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    let mut doc = lopdf::Document::load_mem(&file_bytes)
        .map_err(|e| format!("Failed to parse PDF: {}", e))?;

    if doc.is_encrypted() {
        return Err("PDF is already password-protected".to_string());
    }

    if !doc.trailer.has(b"ID") {
        let u1 = uuid::Uuid::new_v4();
        let u2 = uuid::Uuid::new_v4();
        let file_id_1 = lopdf::Object::String(u1.as_bytes().to_vec(), lopdf::StringFormat::Hexadecimal);
        let file_id_2 = lopdf::Object::String(u2.as_bytes().to_vec(), lopdf::StringFormat::Hexadecimal);
        doc.trailer.set(b"ID", lopdf::Object::Array(vec![file_id_1, file_id_2]));
    }

    let permissions = lopdf::encryption::Permissions::PRINTABLE
        | lopdf::encryption::Permissions::COPYABLE
        | lopdf::encryption::Permissions::COPYABLE_FOR_ACCESSIBILITY
        | lopdf::encryption::Permissions::PRINTABLE_IN_HIGH_QUALITY;

    let requested_version = lopdf::encryption::EncryptionVersion::V2 {
        document: &doc,
        owner_password: &password,
        user_password: &password,
        key_length: 128,
        permissions,
    };

    let state = lopdf::encryption::EncryptionState::try_from(requested_version)
        .map_err(|e| format!("Failed to create encryption state: {:?}", e))?;

    doc.encrypt(&state)
        .map_err(|e| format!("Failed to encrypt document: {:?}", e))?;

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| format!("Failed to save encrypted document: {}", e))?;

    Ok(out)
}

#[tauri::command]
pub async fn unlock_pdf_native(file_bytes: Vec<u8>, password: String) -> Result<Vec<u8>, String> {
    if file_bytes.is_empty() {
        return Err("Cannot unlock empty PDF file".to_string());
    }

    let mut doc = lopdf::Document::load_mem_with_options(
        &file_bytes,
        lopdf::LoadOptions::with_password(&password),
    )
    .or_else(|_| {
        let mut d = lopdf::Document::load_mem(&file_bytes)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;
        d.decrypt(&password)
            .map_err(|e| format!("Incorrect password or decryption error: {:?}", e))?;
        Ok::<lopdf::Document, String>(d)
    })
    .map_err(|e| format!("Failed to unlock PDF: {}", e))?;

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| format!("Failed to save decrypted PDF: {}", e))?;

    Ok(out)
}

