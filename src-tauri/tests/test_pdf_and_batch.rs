use oculus_lib::batch::{BatchEngine, BatchItem};
use oculus_lib::ffi::NativeEngine;
use oculus_lib::models::PreprocessOptions;
use oculus_lib::pdf::PdfPipeline;
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn test_pdf_page_counting_and_rendering() {
    let pdf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(pdf_path).expect("Read sample_digital.pdf");

    // 1. Page count
    let count = NativeEngine::pdf_page_count(&pdf_bytes).expect("Query page count");
    assert_eq!(count, 1, "Sample PDF should have exactly 1 page");

    // 2. High-DPI Page rendering
    let rendered_png = NativeEngine::render_pdf_page(&pdf_bytes, 1, 150).expect("Render page");
    assert!(!rendered_png.is_empty(), "Rendered PNG bytes must not be empty");
    assert!(rendered_png.starts_with(b"\x89PNG"), "Rendered output must be valid PNG");
}

#[test]
fn test_pdf_pipeline_hybrid_processing() {
    let pdf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(pdf_path).expect("Read sample_digital.pdf");
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = AtomicBool::new(false);
    let opts = PreprocessOptions::default();

    let res = PdfPipeline::process_pdf(
        &engine,
        "doc-pdf-hybrid",
        "sample_digital.pdf",
        &pdf_bytes,
        "eng",
        opts,
        &cancel,
        |_curr, _tot, _msg| {},
    )
    .expect("Process hybrid PDF");

    assert_eq!(res.pages.len(), 1);
    assert!(
        res.pages[0].text.contains("OCULUS DIGITAL PDF PAGE EXTRACTION"),
        "Extracted text must contain 'OCULUS DIGITAL PDF PAGE EXTRACTION', got: {}",
        res.pages[0].text
    );
    assert!(res.pages[0].confidence > 80.0, "Confidence should be > 80%");
}

#[tokio::test]
async fn test_batch_processing_engine() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = Arc::new(AtomicBool::new(false));

    let img1 = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let img2 = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_jpg.jpg");
    let pdf1 = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");

    let items = vec![
        BatchItem {
            id: "b-1".to_string(),
            path: img1.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
        BatchItem {
            id: "b-2".to_string(),
            path: img2.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
        BatchItem {
            id: "b-3".to_string(),
            path: pdf1.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
    ];

    let summary = BatchEngine::run_batch(
        engine,
        items,
        2, // 2 concurrent workers
        cancel,
        |_prog| {},
    )
    .await
    .expect("Run batch");

    assert_eq!(summary.total_files, 3);
    assert_eq!(summary.succeeded_files, 3);
    assert_eq!(summary.failed_files, 0);
    assert!(summary.total_pages >= 3);
    assert!(summary.total_duration_ms > 0);
}

#[tokio::test]
async fn test_batch_per_file_error_isolation() {
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = Arc::new(AtomicBool::new(false));

    let valid_img = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let invalid_file = "/non/existent/corrupted_file.xyz";

    let items = vec![
        BatchItem {
            id: "b-ok-1".to_string(),
            path: valid_img.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
        BatchItem {
            id: "b-corrupted".to_string(),
            path: invalid_file.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
        BatchItem {
            id: "b-ok-2".to_string(),
            path: valid_img.to_string(),
            language: "eng".to_string(),
            options: PreprocessOptions::default(),
        },
    ];

    let summary = BatchEngine::run_batch(
        engine,
        items,
        2,
        cancel,
        |_prog| {},
    )
    .await
    .expect("Run batch with error isolation");

    // The batch should complete with 2 successes and 1 failure without panicking
    assert_eq!(summary.total_files, 3);
    assert_eq!(summary.succeeded_files, 2, "Valid files must succeed");
    assert_eq!(summary.failed_files, 1, "Corrupted file must fail gracefully");

    let failed_item = summary.results.iter().find(|r| r.id == "b-corrupted").expect("Find failed item");
    assert!(!failed_item.success);
    assert!(failed_item.error.is_some());
}

#[test]
fn test_pdf_structure_parser_fallback_on_corrupt_structure() {
    // Generate a test PDF with a valid digital PDF header and body
    let pdf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(pdf_path).expect("Read sample_digital.pdf");
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = AtomicBool::new(false);
    let opts = PreprocessOptions::default();

    // Verify NativeEngine pdf_page_count returns valid page count
    let count = NativeEngine::pdf_page_count(&pdf_bytes).expect("Page count");
    assert!(count >= 1);

    // Process PDF and ensure success
    let res = PdfPipeline::process_pdf(
        &engine,
        "doc-pdf-fallback-test",
        "sample_digital.pdf",
        &pdf_bytes,
        "eng",
        opts,
        &cancel,
        |_curr, _tot, _msg| {},
    );
    assert!(res.is_ok(), "PDF pipeline must succeed: {:?}", res.err());
    let ocr = res.expect("Valid OCR result");
    assert_eq!(ocr.pages.len(), count as usize);
}

#[test]
fn test_digital_text_viability_and_fragmentation_fallback() {
    // 1. Digital text viability unit tests
    let clean_text = "This is a clean line of text from a normal document.\nAnother line of proper English words.\nThird sentence with complete words and structure.";
    assert!(oculus_lib::pdf::is_digital_text_viable(clean_text));

    let fragmented_math = "u\n\nz\nz\ne\ne\ny\nD\nD\nD\n0\n1\n3\n8\n+\n-\n=\n(";
    assert!(!oculus_lib::pdf::is_digital_text_viable(fragmented_math));

    let pua_text = "Formula with unmapped private symbols \u{F0B6} \u{F05C} \u{F0BE} \u{F0B6} \u{F05C} \u{F0BE} everywhere";
    assert!(!oculus_lib::pdf::is_digital_text_viable(pua_text));

    // 2. Integration test with sample_digital.pdf
    let pdf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(pdf_path).expect("Read sample_digital.pdf");
    let engine = NativeEngine::new(None).expect("Engine init");
    let cancel = AtomicBool::new(false);
    let opts = PreprocessOptions::default();

    let res = PdfPipeline::process_pdf(
        &engine,
        "doc-digital-viability",
        "sample_digital.pdf",
        &pdf_bytes,
        "eng",
        opts,
        &cancel,
        |_curr, _tot, _msg| {},
    )
    .expect("Process digital PDF");
    assert_eq!(res.pages.len(), 1);
    assert!(res.pages[0].confidence > 90.0);
    assert!(res.pages[0].text.contains("OCULUS DIGITAL PDF PAGE EXTRACTION"));
}

#[tokio::test]
async fn test_native_pdf_repair_protect_and_unlock() {
    let pdf_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(pdf_path).expect("Read sample_digital.pdf");

    // 1. Repair test
    let repaired = oculus_lib::commands::repair_pdf_native(pdf_bytes.clone())
        .await
        .expect("Repair should succeed");
    assert!(!repaired.is_empty());
    assert!(repaired.starts_with(b"%PDF"));

    // 2. Protect test
    let password = "SecretPassword123!".to_string();
    let encrypted = oculus_lib::commands::protect_pdf_native(pdf_bytes.clone(), password.clone())
        .await
        .expect("Protect should succeed");
    assert!(!encrypted.is_empty());
    assert!(encrypted.starts_with(b"%PDF"));

    // Verify it is encrypted
    let doc = lopdf::Document::load_mem(&encrypted).expect("Parse encrypted PDF structure");
    assert!(doc.is_encrypted(), "Output document must be marked encrypted");

    // 3. Unlock test
    let decrypted = oculus_lib::commands::unlock_pdf_native(encrypted, password)
        .await
        .expect("Unlock should succeed with correct password");
    assert!(!decrypted.is_empty());
    assert!(decrypted.starts_with(b"%PDF"));

    let unenc_doc = lopdf::Document::load_mem(&decrypted).expect("Parse decrypted PDF");
    assert!(!unenc_doc.is_encrypted(), "Output document must be decrypted");
}

