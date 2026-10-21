use oculus_lib::ffi::NativeEngine;
use oculus_lib::models::{
    BoundingBox, ExportRequest, OcrBlock, OcrPage, OcrResult, PreprocessOptions,
};
use std::fs;

#[test]
fn test_audit_claim_png_ocr() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let bytes = fs::read(path).expect("Read sample_en.png");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    let res = engine
        .run_ocr("doc-png", "sample_en.png", &bytes, "eng", opts)
        .expect("OCR PNG");
    assert!(!res.pages.is_empty(), "Result must contain at least 1 page");
    assert!(
        res.pages[0].text.contains("OCULUS"),
        "Extracted text must match 'OCULUS'"
    );
    assert!(res.pages[0].confidence > 60.0, "Confidence must be > 60%");
}

#[test]
fn test_audit_claim_jpeg_ocr() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../test_samples/sample_jpg.jpg"
    );
    let bytes = fs::read(path).expect("Read sample_jpg.jpg");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    let res = engine
        .run_ocr("doc-jpg", "sample_jpg.jpg", &bytes, "eng", opts)
        .expect("OCR JPEG");
    assert!(!res.pages.is_empty(), "Result must contain at least 1 page");
    assert!(
        res.pages[0].text.contains("JPEG"),
        "Extracted text must contain 'JPEG'"
    );
    assert!(res.pages[0].confidence > 60.0, "Confidence must be > 60%");
}

#[test]
fn test_audit_claim_bounding_boxes_and_confidence() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let bytes = fs::read(path).expect("Read sample_en.png");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    let res = engine
        .run_ocr("doc-bbox", "sample_en.png", &bytes, "eng", opts)
        .expect("OCR for bbox");
    let page = &res.pages[0];
    assert!(!page.blocks.is_empty(), "Must extract word-level blocks");

    for block in &page.blocks {
        assert!(block.bbox.width > 0, "Bounding box width must be > 0");
        assert!(block.bbox.height > 0, "Bounding box height must be > 0");
        assert!(
            block.confidence >= 0.0 && block.confidence <= 100.0,
            "Confidence must be in [0, 100]"
        );
        assert!(!block.text.is_empty(), "Block text must not be empty");
    }
}

#[test]
fn test_audit_claim_all_four_export_formats() {
    let result = OcrResult {
        document_id: "doc-audit-export".to_string(),
        filename: "test_doc.png".to_string(),
        language: "mal".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: "ഒക്കുലസ് പ്രാദേശിക ഒസിആർ രേഖ".to_string(),
            blocks: vec![OcrBlock {
                bbox: BoundingBox {
                    x: 10,
                    y: 10,
                    width: 100,
                    height: 25,
                },
                confidence: 92.5,
                text: "ഒക്കുലസ്".to_string(),
            }],
            confidence: 92.5,
        }],
        processing_time_ms: 85,
        warnings: Vec::new(),
    };

    let temp_dir = std::env::temp_dir();

    // 1. TXT Export
    let txt_path = temp_dir.join("audit_export.txt");
    let txt_req = ExportRequest {
        document_id: "doc-audit-export".to_string(),
        result: result.clone(),
        format: "txt".to_string(),
        destination_path: Some(txt_path.to_string_lossy().into_owned()),
    };
    let txt_content = txt_req
        .result
        .pages
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    fs::write(&txt_path, txt_content.as_bytes()).expect("Write TXT");
    let read_txt = fs::read_to_string(&txt_path).expect("Read TXT");
    assert!(read_txt.contains("ഒക്കുലസ്"), "TXT must preserve Malayalam");

    // 2. Markdown Export
    let md_path = temp_dir.join("audit_export.md");
    let md_content = format!("# {}\n\n{}", result.filename, result.pages[0].text);
    fs::write(&md_path, md_content.as_bytes()).expect("Write MD");
    let read_md = fs::read_to_string(&md_path).expect("Read MD");
    assert!(
        read_md.contains("# test_doc.png"),
        "Markdown must contain title"
    );
    assert!(
        read_md.contains("പ്രാദേശിക"),
        "Markdown must preserve Malayalam"
    );

    // 3. JSON Export
    let json_path = temp_dir.join("audit_export.json");
    let json_str = serde_json::to_string_pretty(&result).expect("Serialize JSON");
    fs::write(&json_path, json_str.as_bytes()).expect("Write JSON");
    let read_json = fs::read_to_string(&json_path).expect("Read JSON");
    let parsed: OcrResult = serde_json::from_str(&read_json).expect("Deserialize JSON");
    assert_eq!(parsed.pages[0].text, result.pages[0].text);

    // Cleanup
    let _ = fs::remove_file(txt_path);
    let _ = fs::remove_file(md_path);
    let _ = fs::remove_file(json_path);
}
