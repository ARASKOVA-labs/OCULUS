use oculus_lib::ffi::NativeEngine;
use oculus_lib::intelligence::DocumentIntelligence;
use oculus_lib::models::PreprocessOptions;
use std::fs;
use unicode_normalization::UnicodeNormalization;

fn has_malayalam_chars(s: &str) -> bool {
    s.chars().any(|c| ('\u{0D00}'..='\u{0D7F}').contains(&c))
}

#[test]
fn test_malayalam_chillus() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_chillus.png");
    let bytes = fs::read(path).expect("Read chillus image");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    let res = engine.run_ocr("doc-chillus", "mal_chillus.png", &bytes, "mal", opts).expect("OCR Chillus");
    assert!(!res.pages.is_empty(), "Must have pages");
    let text = &res.pages[0].text;
    assert!(has_malayalam_chars(text), "Must extract Malayalam characters");

    // Test Unicode NFC normalization
    let normalized: String = text.nfc().collect();
    assert_eq!(text, &normalized, "Text should already be NFC normalized Unicode");
}

#[test]
fn test_malayalam_conjuncts() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_conjuncts.png");
    let bytes = fs::read(path).expect("Read conjuncts image");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions {
        grayscale: true,
        denoise: true,
        deskew: false,
        threshold: true,
        orientation_detect: false,
        contrast_norm: false,
    };

    let res = engine.run_ocr("doc-conjuncts", "mal_conjuncts.png", &bytes, "mal", opts).expect("OCR Conjuncts");
    assert!(!res.pages.is_empty());
    let text = &res.pages[0].text;
    assert!(has_malayalam_chars(text));
}

#[test]
fn test_malayalam_mixed_bilingual() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_mixed.png");
    let bytes = fs::read(path).expect("Read mixed image");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions::default();

    // Use "eng+mal" or "mal"
    let res = engine.run_ocr("doc-mixed", "mal_mixed.png", &bytes, "mal", opts).expect("OCR Mixed");
    assert!(!res.pages.is_empty());
    let text = &res.pages[0].text;
    assert!(has_malayalam_chars(text));
}

#[test]
fn test_malayalam_receipt_and_intelligence() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_receipt.png");
    let bytes = fs::read(path).expect("Read receipt image");
    let engine = NativeEngine::new(None).expect("Engine init");
    let opts = PreprocessOptions {
        grayscale: true,
        denoise: false,
        deskew: false,
        threshold: true,
        orientation_detect: false,
        contrast_norm: true,
    };

    let res = engine.run_ocr("doc-receipt", "mal_receipt.png", &bytes, "mal", opts).expect("OCR Receipt");
    assert!(!res.pages.is_empty());
    let text = &res.pages[0].text;
    assert!(has_malayalam_chars(text));

    // Test Document Intelligence field extraction
    let structured = DocumentIntelligence::analyze(&res);
    assert_eq!(structured.language, "mal");
}

#[test]
fn test_malayalam_searchable_pdf_generation() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_mixed.png");
    let bytes = fs::read(path).expect("Read mixed image");
    let engine = NativeEngine::new(None).expect("Engine init");

    let temp_dir = std::env::temp_dir();
    let out_base = temp_dir.join("test_searchable_mal");
    let out_base_str = out_base.to_string_lossy().into_owned();

    let pdf_path = temp_dir.join("test_searchable_mal.pdf");
    let _ = fs::remove_file(&pdf_path);

    let res = engine.create_searchable_pdf(&bytes, "mal", &out_base_str);
    assert!(res.is_ok(), "Searchable PDF creation should succeed");
    assert!(pdf_path.exists(), "Searchable PDF file must be written to disk");

    let metadata = fs::metadata(&pdf_path).expect("Read PDF metadata");
    assert!(metadata.len() > 100, "Searchable PDF must have valid size");

    let _ = fs::remove_file(&pdf_path);
}
