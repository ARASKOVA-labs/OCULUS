use oculus_lib::ffi::NativeEngine;
use oculus_lib::models::{BoundingBox, OcrBlock, OcrPage, OcrResult, PreprocessOptions};

#[test]
fn test_native_engine_init() {
    let engine = NativeEngine::new(None);
    assert!(engine.is_ok(), "Native engine should initialize with default tessdata");
}

#[test]
fn test_preprocessing_invalid_data() {
    let empty = [];
    let opts = PreprocessOptions::default();
    let res = NativeEngine::preprocess_image(&empty, opts);
    assert!(res.is_err(), "Empty data should return an error");
}

#[test]
fn test_ocr_invalid_buffer() {
    let engine = NativeEngine::new(None).expect("engine init");
    let empty = [];
    let opts = PreprocessOptions::default();
    let res = engine.run_ocr("doc-1", "test.png", &empty, "eng", opts);
    assert!(res.is_err(), "Empty buffer should return error");
}

#[test]
fn test_malayalam_utf8_preservation_in_models_and_json() {
    // Verified Malayalam text: "ഒക്കുലസ് - പ്രാദേശിക ഒസിആർ" (OCULUS - Local OCR)
    let mal_text = "ഒക്കുലസ് - പ്രാദേശിക ഒസിആർ";

    let result = OcrResult {
        document_id: "doc-mal-1".to_string(),
        filename: "malayalam_doc.png".to_string(),
        language: "mal".to_string(),
        pages: vec![OcrPage {
            page: 1,
            text: mal_text.to_string(),
            blocks: vec![OcrBlock {
                bbox: BoundingBox {
                    x: 10,
                    y: 20,
                    width: 150,
                    height: 30,
                },
                confidence: 94.2,
                text: mal_text.to_string(),
            }],
            confidence: 94.2,
        }],
        processing_time_ms: 120,
        warnings: Vec::new(),
    };

    // Verify JSON serialization maintains UTF-8 characters without escapes or loss
    let json = serde_json::to_string(&result).expect("JSON serialization");
    assert!(json.contains("ഒക്കുലസ്"), "JSON must preserve Malayalam text");

    let deserialized: OcrResult = serde_json::from_str(&json).expect("JSON deserialization");
    assert_eq!(
        deserialized.pages[0].text, mal_text,
        "Malayalam text must roundtrip identically through serde"
    );

    // Verify JSONL line serialization
    let jsonl_line = serde_json::json!({
        "document": result.filename,
        "page": 1,
        "confidence": result.pages[0].confidence,
        "language": result.language,
        "text": result.pages[0].text,
    });
    let line_str = serde_json::to_string(&jsonl_line).expect("JSONL serialization");
    assert!(line_str.contains("പ്രാദേശിക"), "JSONL must preserve Malayalam text");
}

#[test]
fn test_ocr_on_synthesized_image() {
    // Generate a minimal 200x50 PNG with white background and black rectangle
    let mut img = image::RgbImage::new(200, 60);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 255, 255]);
    }
    // Draw simple dark pattern
    for x in 20..180 {
        for y in 20..40 {
            img.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }
    }

    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )
    .expect("write png");

    let engine = NativeEngine::new(None).expect("engine init");
    let opts = PreprocessOptions {
        grayscale: true,
        denoise: false,
        deskew: false,
        threshold: true,
        orientation_detect: false,
        contrast_norm: false,
    };

    // Test preprocessing
    let prep_res = NativeEngine::preprocess_image(&png_bytes, opts);
    assert!(prep_res.is_ok(), "Preprocessing should succeed on valid PNG");
    assert!(!prep_res.expect("prep bytes").is_empty());

    // Test OCR execution
    let ocr_res = engine.run_ocr("synth-1", "synth.png", &png_bytes, "eng", opts);
    assert!(ocr_res.is_ok(), "OCR should complete without panic on synthetic image");
    let res = ocr_res.expect("ocr result");
    assert_eq!(res.document_id, "synth-1");
    assert_eq!(res.pages.len(), 1);
}

#[test]
fn test_real_english_ocr() {
    let sample_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    if let Ok(bytes) = std::fs::read(sample_path) {
        let engine = NativeEngine::new(None).expect("engine init");
        let opts = PreprocessOptions {
            grayscale: true,
            denoise: false,
            deskew: false,
            threshold: true,
            orientation_detect: false,
            contrast_norm: false,
        };
        let res = engine.run_ocr("doc-en", "sample_en.png", &bytes, "eng", opts)
            .expect("english ocr");
        assert!(!res.pages.is_empty(), "must have pages");
        let text = &res.pages[0].text;
        assert!(text.contains("OCULUS"), "extracted text should contain 'OCULUS', got: {}", text);
        assert!(res.pages[0].confidence > 50.0, "confidence should be > 50%, got: {}", res.pages[0].confidence);
    }
}

#[test]
fn test_real_malayalam_ocr() {
    let sample_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_mal.png");
    if let Ok(bytes) = std::fs::read(sample_path) {
        let engine = NativeEngine::new(None).expect("engine init");
        let opts = PreprocessOptions {
            grayscale: true,
            denoise: false,
            deskew: false,
            threshold: true,
            orientation_detect: false,
            contrast_norm: false,
        };
        let res = engine.run_ocr("doc-mal", "sample_mal.png", &bytes, "mal", opts)
            .expect("malayalam ocr");
        assert!(!res.pages.is_empty(), "must have pages");
        let text = &res.pages[0].text;
        // Verify Malayalam Unicode block presence (U+0D00 to U+0D7F)
        let has_malayalam = text.chars().any(|c| ('\u{0D00}'..='\u{0D7F}').contains(&c));
        assert!(has_malayalam, "extracted text must contain Malayalam Unicode characters, got: {}", text);
    }
}
