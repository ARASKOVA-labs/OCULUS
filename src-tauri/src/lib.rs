pub mod batch;
pub mod commands;
pub mod ffi;
pub mod intelligence;
pub mod models;
pub mod pdf;

use commands::AppState;
use ffi::NativeEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn run() {
    let engine = NativeEngine::new(None).expect("Failed to initialize native OCR engine");
    let state = AppState {
        engine,
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::inspect_document,
            commands::preprocess_image,
            commands::run_ocr,
            commands::cancel_ocr,
            commands::start_batch_ocr,
            commands::analyze_document,
            commands::export_result,
            commands::render_pdf_page,
            commands::read_binary_file,
            commands::show_in_folder,
            commands::repair_pdf_native,
            commands::protect_pdf_native,
            commands::unlock_pdf_native
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
