use oculus_lib::batch::{BatchEngine, BatchItem};
use oculus_lib::ffi::NativeEngine;
use oculus_lib::intelligence::DocumentIntelligence;
use oculus_lib::models::{OcrBlock, OcrPage, OcrResult, PreprocessOptions};
use oculus_lib::pdf::PdfPipeline;
use std::fs;
use std::io::Cursor;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;

#[test]
fn test_chaos_malformed_truncated_pdf() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = AtomicBool::new(false);
    let opts = PreprocessOptions::default();

    // 1. Truncated header
    let truncated_bytes = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\n";
    let res = PdfPipeline::process_pdf(
        &engine,
        "chaos-trunc-pdf",
        "truncated.pdf",
        truncated_bytes,
        "eng",
        opts,
        &cancel,
        |_c, _t, _m| {},
    );
    assert!(res.is_err(), "Malformed truncated PDF must return an Err, not panic");

    // 2. Garbage bytes with PDF magic
    let garbage_pdf = b"%PDF-corrupted-random-garbage-data-here-1234567890";
    let res_garbage = PdfPipeline::process_pdf(
        &engine,
        "chaos-garbage-pdf",
        "garbage.pdf",
        garbage_pdf,
        "eng",
        opts,
        &cancel,
        |_c, _t, _m| {},
    );
    assert!(res_garbage.is_err(), "Garbage PDF bytes must return an Err");
}

#[test]
fn test_chaos_empty_zero_byte_pdf() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = AtomicBool::new(false);
    let opts = PreprocessOptions::default();

    let empty_bytes: &[u8] = &[];
    let res = PdfPipeline::process_pdf(
        &engine,
        "chaos-empty-pdf",
        "empty.pdf",
        empty_bytes,
        "eng",
        opts,
        &cancel,
        |_c, _t, _m| {},
    );
    assert!(res.is_err(), "0-byte PDF must fail cleanly");
    assert_eq!(res.unwrap_err(), "Cannot process empty PDF document");

    // Also verify native CoreGraphics page count and render with empty slice
    assert!(NativeEngine::pdf_page_count(empty_bytes).is_err());
    assert!(NativeEngine::render_pdf_page(empty_bytes, 1, 300).is_err());
}

#[test]
fn test_chaos_malformed_truncated_images() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    // 1. Truncated PNG header
    let truncated_png = b"\x89PNG\r\n\x1a\n";
    let res = engine.run_ocr("chaos-trunc-png", "trunc.png", truncated_png, "eng", opts);
    assert!(res.is_err(), "Truncated PNG must return an error without panicking");

    // 2. Corrupted JPEG SOI header only
    let corrupt_jpeg = b"\xFF\xD8\xFF\xE0\x00\x10JFIF";
    let res_jpeg = engine.run_ocr("chaos-corrupt-jpg", "corrupt.jpg", corrupt_jpeg, "eng", opts);
    assert!(res_jpeg.is_err(), "Corrupt JPEG must fail gracefully");

    // 3. Preprocessing on corrupt image
    let res_prep = NativeEngine::preprocess_image(truncated_png, opts);
    assert!(res_prep.is_err(), "Preprocessing truncated PNG must fail cleanly");
}

#[test]
fn test_chaos_extreme_dimension_image() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions {
        grayscale: true,
        denoise: false,
        contrast_norm: false,
        threshold: true,
        deskew: false,
        orientation_detect: false,
    };

    // Create a large 3500x3500 image with text
    let width = 3500;
    let height = 3500;
    let mut img = image::RgbImage::from_pixel(width, height, image::Rgb([255, 255, 255]));

    // Draw some black pixels in the center
    for y in 1700..1800 {
        for x in 1700..1800 {
            img.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }
    }

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png).expect("Encode large PNG");

    assert!(png_bytes.len() > 10_000, "Large PNG must be encoded");

    // 1. Preprocess extreme image
    let prep_res = NativeEngine::preprocess_image(&png_bytes, opts);
    assert!(prep_res.is_ok(), "Preprocessing large 3500x3500 image should succeed without OOM");

    // 2. Run OCR on extreme image
    let ocr_res = engine.run_ocr("chaos-large-img", "large.png", &png_bytes, "eng", opts);
    assert!(ocr_res.is_ok(), "OCR on large 3500x3500 image should complete safely");
}

#[test]
fn test_chaos_unusual_unicode_and_malayalam_edge_cases() {
    // Test complex Malayalam Unicode including zero-width joiners (ZWJ), non-joiners (ZWNJ),
    // and archaic numerals
    let text_with_zwj = "അ\u{200D}പൂ\u{200C}ർണ്ണത ൰ ൱ ൲ ൳ ൴ ൵";
    let normalized: String = text_with_zwj.nfc().collect();
    assert!(!normalized.is_empty(), "NFC normalization must succeed");

    let page_text = format!("വിവരം: {}", normalized);
    let ocr_result = OcrResult {
        document_id: "chaos-unicode".to_string(),
        filename: "unicode_test.png".to_string(),
        language: "mal".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: page_text,
            blocks: vec![
                OcrBlock {
                    bbox: oculus_lib::models::BoundingBox {
                        x: 10,
                        y: 10,
                        width: 100,
                        height: 20,
                    },
                    confidence: 95.0,
                    text: "വിവരം:".to_string(),
                },
                OcrBlock {
                    bbox: oculus_lib::models::BoundingBox {
                        x: 120,
                        y: 10,
                        width: 200,
                        height: 20,
                    },
                    confidence: 90.0,
                    text: normalized.clone(),
                },
            ],
            confidence: 92.5,
        }],
        processing_time_ms: 50,
        warnings: vec![],
    };

    let structured = DocumentIntelligence::analyze(&ocr_result);
    assert!(!structured.fields.is_empty(), "Should extract key-value with Malayalam separator");
    assert!(structured.fields.contains_key("വിവരം"));

    // Test JSON serialization of edge-case Unicode
    let json = serde_json::to_string(&ocr_result).expect("Serialize to JSON");
    assert!(json.contains("വിവരം"), "JSON must preserve raw UTF-8 characters");
    let deserialized: OcrResult = serde_json::from_str(&json).expect("Deserialize from JSON");
    assert_eq!(deserialized.pages[0].blocks[1].text, normalized);
}

#[tokio::test]
async fn test_chaos_batch_massive_queue_and_duplicates() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = Arc::new(AtomicBool::new(false));
    let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");

    // Create 50 batch items with duplicate paths
    let mut items = Vec::new();
    for i in 0..50 {
        items.push(BatchItem {
            id: format!("batch-item-{}", i),
            path: img_path.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        });
    }

    let summary = BatchEngine::run_batch(
        engine,
        items,
        4, // 4 concurrent workers
        cancel,
        |_ev| {},
    )
    .await
    .expect("Massive batch processing");

    assert_eq!(summary.total_files, 50);
    assert_eq!(summary.succeeded_files, 50);
    assert_eq!(summary.failed_files, 0);
}

#[tokio::test]
async fn test_chaos_batch_immediate_cancellation() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = Arc::new(AtomicBool::new(true)); // Pre-cancelled!
    let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");

    let items = vec![BatchItem {
        id: "cancel-item-1".to_string(),
        path: img_path.to_string(),
        language: "eng".to_string(),
        options: PreprocessOptions::default(),
    }];

    let summary = BatchEngine::run_batch(engine, items, 2, cancel, |_ev| {})
        .await
        .expect("Cancelled batch run");

    assert_eq!(summary.failed_files, 1, "Batch summary must report 1 failed file on cancel");
    assert_eq!(
        summary.results[0].error.as_deref(),
        Some("Job cancelled by user")
    );
}

#[test]
fn test_chaos_export_searchable_pdf_from_image_source() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let img_bytes = fs::read(img_path).expect("Read sample_en.png");

    // Demonstrating the weakness: passing image bytes to render_pdf_page fails
    let render_res = NativeEngine::render_pdf_page(&img_bytes, 1, 300);
    assert!(
        render_res.is_err(),
        "CoreGraphics render_pdf_page cannot render a PNG file as a PDF (expected error)"
    );

    // But create_searchable_pdf expects raster image bytes directly!
    let out_dir = std::env::temp_dir().join(format!("oculus_searchable_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&out_dir).expect("Create temp dir");
    let output_base = out_dir.join("image_searchable");

    let pdf_res = engine.create_searchable_pdf(
        &img_bytes,
        "eng",
        &output_base.to_string_lossy(),
    );
    assert!(
        pdf_res.is_ok(),
        "create_searchable_pdf should succeed directly with image bytes without going through render_pdf_page"
    );
    let final_pdf = out_dir.join("image_searchable.pdf");
    assert!(final_pdf.exists(), "Searchable PDF should be written to disk");
}

#[test]
fn test_chaos_concurrent_engine_invocations() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let bytes = Arc::new(fs::read(img_path).expect("Read sample_en.png"));

    let mut handles = Vec::new();
    for i in 0..8 {
        let eng = engine.clone();
        let b = bytes.clone();
        let handle = std::thread::spawn(move || {
            let doc_id = format!("concurrent-{}", i);
            eng.run_ocr(
                &doc_id,
                "sample_en.png",
                &b,
                "eng",
                PreprocessOptions::default(),
            )
        });
        handles.push(handle);
    }

    for (i, h) in handles.into_iter().enumerate() {
        let res = h.join().expect("Thread joined successfully");
        assert!(res.is_ok(), "Concurrent OCR invocation #{} failed: {:?}", i, res.err());
        let ocr_res = res.expect("Valid OCR result from thread");
        assert!(ocr_res.pages[0].text.contains("OCULUS"));
    }
}

#[test]
fn test_chaos_structured_extraction_varied_separators() {
    // Testing multiple real-world key-value separators (colon, full-width colon, equals, dash)
    let full_text = "പേര്: അർജുൻ\nരജിസ്ട്രേഷൻ：KL-07-2026\nആകെ തുക = 4500\nസ്ഥലം - എറണാകുളം";
    let ocr_result = OcrResult {
        document_id: "separators-test".to_string(),
        filename: "test.png".to_string(),
        language: "mal".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: full_text.to_string(),
            blocks: vec![],
            confidence: 95.0,
        }],
        processing_time_ms: 10,
        warnings: vec![],
    };

    let structured = DocumentIntelligence::analyze(&ocr_result);
    // Standard colon
    assert!(structured.fields.contains_key("പേര്"), "Must extract standard colon key");
    assert_eq!(structured.fields.get("പേര്").expect("Field exists"), "അർജുൻ");
    // Dash separator
    assert!(structured.fields.contains_key("സ്ഥലം"), "Must extract dash separator key");
    assert_eq!(structured.fields.get("സ്ഥലം").expect("Field exists"), "എറണാകുളം");
    // Full-width colon
    assert!(structured.fields.contains_key("രജിസ്ട്രേഷൻ"), "Must extract full-width colon key");
    assert_eq!(structured.fields.get("രജിസ്ട്രേഷൻ").expect("Field exists"), "KL-07-2026");
    // Equals sign
    assert!(structured.fields.contains_key("ആകെ തുക"), "Must extract equals sign key");
    assert_eq!(structured.fields.get("ആകെ തുക").expect("Field exists"), "4500");
}

#[test]
fn test_chaos_perform_export_and_directory_creation() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let ocr_res = OcrResult {
        document_id: "doc-export-chaos".to_string(),
        filename: "test_doc.png".to_string(),
        language: "eng".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: "Hello OCULUS Export".to_string(),
            blocks: vec![],
            confidence: 99.0,
        }],
        processing_time_ms: 12,
        warnings: vec![],
    };

    // Export to deeply nested non-existent directory
    let nested_dir = std::env::temp_dir()
        .join(format!("oculus_deep_dir_{}", uuid::Uuid::new_v4()))
        .join("level1")
        .join("level2");
    let target_file = nested_dir.join("export.txt");

    let req = oculus_lib::models::ExportRequest {
        document_id: "doc-export-chaos".to_string(),
        format: "txt".to_string(),
        result: ocr_res.clone(),
        destination_path: Some(target_file.to_string_lossy().into_owned()),
    };

    let export_res = oculus_lib::commands::perform_export(&engine, req);
    assert!(export_res.is_ok(), "perform_export should create parent directories automatically");
    assert!(target_file.exists(), "Exported file must exist on disk");
    let content = fs::read_to_string(&target_file).expect("Read exported file");
    assert_eq!(content, "Hello OCULUS Export");

    // Test Searchable PDF export from image file
    let img_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let mut img_ocr = ocr_res.clone();
    img_ocr.filename = img_path.to_string();

    let pdf_target = nested_dir.join("image_searchable.pdf");
    let pdf_req = oculus_lib::models::ExportRequest {
        document_id: "doc-export-chaos".to_string(),
        format: "pdf".to_string(),
        result: img_ocr,
        destination_path: Some(pdf_target.to_string_lossy().into_owned()),
    };

    let pdf_export_res = oculus_lib::commands::perform_export(&engine, pdf_req);
    assert!(
        pdf_export_res.is_ok(),
        "Exporting Searchable PDF from an image source should succeed: {:?}",
        pdf_export_res.err()
    );
    assert!(pdf_target.exists(), "Searchable PDF must be created");
    assert!(pdf_target.metadata().expect("PDF metadata").len() > 1000);
}

#[test]
fn test_export_default_destination_never_writes_to_root() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let ocr_res = OcrResult {
        document_id: "doc-root-safety".to_string(),
        filename: "ST-Agency-P-0001.pdf".to_string(),
        language: "eng".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: "Root safety check text".to_string(),
            blocks: vec![],
            confidence: 99.5,
        }],
        processing_time_ms: 5,
        warnings: vec![],
    };

    // Case 1: destination_path is None (default export)
    let req_default = oculus_lib::models::ExportRequest {
        document_id: "doc-root-safety".to_string(),
        format: "json".to_string(),
        result: ocr_res.clone(),
        destination_path: None,
    };

    let res_default = oculus_lib::commands::perform_export(&engine, req_default)
        .expect("Default export must succeed without hitting read-only root filesystem");
    assert!(res_default.success);
    let path = std::path::Path::new(&res_default.destination_path);
    assert_ne!(
        path.parent(),
        Some(std::path::Path::new("/")),
        "Default export must NEVER target the read-only root filesystem '/'"
    );
    assert!(path.exists(), "Exported file must exist at {}", path.display());
    let _ = fs::remove_file(path);

    // Case 2: destination_path explicitly set to root directory path (e.g. "/ST-Agency-P-0001_extracted.txt")
    let req_root_explicit = oculus_lib::models::ExportRequest {
        document_id: "doc-root-safety".to_string(),
        format: "txt".to_string(),
        result: ocr_res,
        destination_path: Some("/ST-Agency-P-0001_extracted.txt".to_string()),
    };

    let res_explicit = oculus_lib::commands::perform_export(&engine, req_root_explicit)
        .expect("Explicit root path must be safely redirected to a writable directory");
    assert!(res_explicit.success);
    let path_explicit = std::path::Path::new(&res_explicit.destination_path);
    assert_ne!(
        path_explicit.parent(),
        Some(std::path::Path::new("/")),
        "Explicit root path must be redirected away from root '/'"
    );
    assert!(
        path_explicit.exists(),
        "Redirected file must exist at {}",
        path_explicit.display()
    );
    let _ = fs::remove_file(path_explicit);
}

