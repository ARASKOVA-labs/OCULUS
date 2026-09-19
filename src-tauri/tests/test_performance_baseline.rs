use oculus_lib::batch::{BatchEngine, BatchItem};
use oculus_lib::ffi::NativeEngine;
use oculus_lib::models::{ExportRequest, PreprocessOptions};
use oculus_lib::pdf::PdfPipeline;
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_measure_performance_baseline() {
    println!("\n=== OCULUS PERFORMANCE BENCHMARK EXECUTION ===");

    // 1. Engine Cold Startup Latency
    let t0 = Instant::now();
    let engine = NativeEngine::new(None).expect("Engine init");
    let startup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("1. Native Engine Startup: {:.2} ms", startup_ms);

    // 2. Preprocessing Overhead
    let sample_en = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_en.png");
    let en_bytes = fs::read(sample_en).expect("Read sample_en.png");

    let t_prep0 = Instant::now();
    let opts = PreprocessOptions {
        grayscale: true,
        denoise: true,
        contrast_norm: true,
        threshold: true,
        deskew: true,
        orientation_detect: false,
    };
    let preprocessed_bytes = NativeEngine::preprocess_image(&en_bytes, opts).expect("Preprocess");
    let prep_overhead_ms = t_prep0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "2. 6-Stage Preprocessing Overhead: {:.2} ms (input: {} bytes, output: {} bytes)",
        prep_overhead_ms,
        en_bytes.len(),
        preprocessed_bytes.len()
    );

    // 3. Single-Image English OCR
    let t_en0 = Instant::now();
    let res_en = engine
        .run_ocr(
            "bench-en",
            sample_en,
            &en_bytes,
            "eng",
            PreprocessOptions::default(),
        )
        .expect("English OCR");
    let single_img_en_ms = t_en0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "3. Single-Image English OCR: {:.2} ms (confidence: {:.1}%, text length: {} chars)",
        single_img_en_ms,
        res_en.pages[0].confidence,
        res_en.pages[0].text.len()
    );

    // 4. Single-Image Malayalam OCR
    let sample_mal = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/corpus/malayalam/mal_conjuncts.png");
    let mal_bytes = fs::read(sample_mal).expect("Read mal_conjuncts.png");

    let t_mal0 = Instant::now();
    let res_mal = engine
        .run_ocr(
            "bench-mal",
            "mal_conjuncts.png",
            &mal_bytes,
            "mal",
            PreprocessOptions::default(),
        )
        .expect("Malayalam OCR");
    let single_img_mal_ms = t_mal0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "4. Single-Image Malayalam OCR: {:.2} ms (confidence: {:.1}%, text length: {} chars)",
        single_img_mal_ms,
        res_mal.pages[0].confidence,
        res_mal.pages[0].text.len()
    );

    // 5. PDF 300 DPI Rendering & OCR Pipeline
    let sample_pdf = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_samples/sample_digital.pdf");
    let pdf_bytes = fs::read(sample_pdf).expect("Read sample_digital.pdf");

    let t_render0 = Instant::now();
    let rendered_png = NativeEngine::render_pdf_page(&pdf_bytes, 1, 300).expect("Render PDF");
    let pdf_render_ms = t_render0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "5a. CoreGraphics 300 DPI PDF Rasterization: {:.2} ms (rendered PNG size: {} bytes)",
        pdf_render_ms,
        rendered_png.len()
    );

    let cancel = AtomicBool::new(false);
    let t_pdf_full = Instant::now();
    let res_pdf = PdfPipeline::process_pdf(
        &engine,
        "bench-pdf",
        "sample_digital.pdf",
        &pdf_bytes,
        "eng",
        PreprocessOptions::default(),
        &cancel,
        |_c, _t, _m| {},
    )
    .expect("PDF pipeline");
    let pdf_total_ms = t_pdf_full.elapsed().as_secs_f64() * 1000.0;
    println!(
        "5b. Full PDF Hybrid Pipeline: {:.2} ms (pages: {})",
        pdf_total_ms,
        res_pdf.pages.len()
    );

    // 6. Searchable PDF Generation
    let out_dir = std::env::temp_dir().join(format!("oculus_bench_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&out_dir).expect("Create bench temp dir");
    let pdf_target = out_dir.join("bench_searchable.pdf");

    let export_req = ExportRequest {
        document_id: "bench-searchable".to_string(),
        format: "pdf".to_string(),
        result: res_en.clone(),
        destination_path: Some(pdf_target.to_string_lossy().into_owned()),
    };

    let t_spdf0 = Instant::now();
    let export_res = oculus_lib::commands::perform_export(&engine, export_req).expect("Searchable PDF export");
    let searchable_pdf_ms = t_spdf0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "6. Searchable PDF Generation: {:.2} ms (bytes written: {})",
        searchable_pdf_ms,
        export_res.bytes_written
    );

    // 7. Cancellation Latency
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let t_cancel0 = Instant::now();
    cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    let cancel_latency_us = t_cancel0.elapsed().as_micros();
    println!("7. Atomic Cancellation Response Latency: {} microseconds", cancel_latency_us);

    // 8. Batch Throughput Measurement (10 items, 4 concurrent workers)
    let tokio_rt = tokio::runtime::Runtime::new().expect("Tokio runtime");
    let summary = tokio_rt.block_on(async {
        let items: Vec<BatchItem> = (0..10)
            .map(|i| BatchItem {
                id: format!("bench-item-{}", i),
                path: sample_en.to_string(),
                language: "eng".to_string(),
                options: PreprocessOptions::default(),
            })
            .collect();

        let cancel_tok = Arc::new(AtomicBool::new(false));
        let t_batch0 = Instant::now();
        let s = BatchEngine::run_batch(
            engine.clone(),
            items,
            4,
            cancel_tok,
            |_p| {},
        )
        .await
        .expect("Batch run");
        let batch_ms = t_batch0.elapsed().as_secs_f64() * 1000.0;
        (s, batch_ms)
    });

    let (batch_summary, batch_ms) = summary;
    let pages_per_second = (batch_summary.total_pages as f64) / (batch_ms / 1000.0);
    println!(
        "8. Batch Processing Throughput: {:.2} ms for 10 documents ({:.2} pages/sec with 4 workers)",
        batch_ms,
        pages_per_second
    );

    println!("===============================================\n");
}
