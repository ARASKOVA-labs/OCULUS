pub mod batch;
pub mod commands;
pub mod ffi;
pub mod intelligence;
pub mod models;
pub mod pdf;
pub mod widget;

use commands::AppState;
use ffi::NativeEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{Manager, WindowEvent};

pub fn run() {
    let engine = NativeEngine::new(None).expect("Failed to initialize native OCR engine");
    let state = AppState {
        engine,
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            let handle = app.handle().clone();
            // macOS opens to the drop widget (the full window is one click away); elsewhere the
            // window opens as before, until the floating widget is tested there
            if cfg!(target_os = "macos") {
                widget::create(&handle)?;
            } else {
                widget::show_main(&handle);
            }
            Ok(())
        })
        .on_menu_event(|app, event| widget::on_menu(app, event.id().as_ref()))
        .on_window_event(|window, event| {
            // with the widget on screen, closing the full window only hides it
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main"
                    && window.app_handle().get_webview_window(widget::LABEL).is_some()
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
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
            commands::save_beside,
            commands::repair_pdf_native,
            commands::protect_pdf_native,
            commands::unlock_pdf_native,
            widget::widget_open,
            widget::widget_menu,
            widget::widget_ready
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            tauri::RunEvent::Exit => commands::guard::clean_up(),
            // clicking the Dock icon opens the full window
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => widget::show_main(app),
            _ => {
                let _ = app;
            }
        });
}
