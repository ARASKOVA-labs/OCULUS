use crate::ffi::{NativeEngine, PDF_RASTER_UNSUPPORTED};
use crate::models::{BoundingBox, OcrBlock, OcrPage, OcrResult, PreprocessOptions};
use lopdf::Document;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub struct PdfPipeline;

pub fn is_digital_text_viable(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    let non_whitespace_count = trimmed.chars().filter(|c| !c.is_whitespace()).count();
    if non_whitespace_count < 30 {
        return false;
    }

    let non_empty_lines: Vec<&str> = trimmed
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if non_empty_lines.is_empty() {
        return false;
    }

    // 1. Check for vertical character stacking / extreme line fragmentation
    let short_line_count = non_empty_lines
        .iter()
        .filter(|l| l.chars().count() <= 2)
        .count();
    let short_ratio = short_line_count as f32 / non_empty_lines.len() as f32;

    // If more than 20% of lines have 1-2 chars when there are at least 5 lines, layout is broken.
    if non_empty_lines.len() >= 5 && short_ratio > 0.20 {
        return false;
    }

    // 2. Average line length check: normal paragraphs have average length >= 10 chars.
    let total_chars: usize = non_empty_lines.iter().map(|l| l.chars().count()).sum();
    let avg_line_len = total_chars as f32 / non_empty_lines.len() as f32;
    if non_empty_lines.len() >= 5 && avg_line_len < 10.0 {
        return false;
    }

    // 3. Check for unmapped Private Use Area (PUA) symbols (\u{E000}..\u{F8FF}) or unprintable control chars
    let unprintable_count = trimmed
        .chars()
        .filter(|&c| {
            (c >= '\u{E000}' && c <= '\u{F8FF}') || (c < ' ' && c != '\n' && c != '\r' && c != '\t')
        })
        .count();
    let unprintable_ratio = unprintable_count as f32 / trimmed.chars().count() as f32;
    if unprintable_ratio > 0.03 {
        // More than 3% unmapped or private-use characters
        return false;
    }

    true
}

pub fn normalize_digital_text(text: &str) -> String {
    let mut cleaned_lines = Vec::new();
    let mut consecutive_empty = 0;

    for line in text.lines() {
        let sanitized: String = line
            .chars()
            .filter(|&c| {
                (c >= ' ' || c == '\t') && !(c >= '\u{E000}' && c <= '\u{F8FF}') && c != '\u{007F}'
            })
            .collect();
        let trimmed = sanitized.trim();

        if trimmed.is_empty() {
            consecutive_empty += 1;
            if consecutive_empty <= 1 && !cleaned_lines.is_empty() {
                cleaned_lines.push(String::new());
            }
        } else {
            consecutive_empty = 0;
            cleaned_lines.push(trimmed.to_string());
        }
    }

    cleaned_lines.join("\n")
}

impl PdfPipeline {
    /// Process a multi-page PDF document using hybrid extraction:
    /// - High-quality digital text pages: extract and normalize existing text directly.
    /// - Scanned image or fragmented math pages: render at 300 DPI, preprocess, and run OCR.
    #[allow(clippy::too_many_arguments)]
    pub fn process_pdf<F>(
        engine: &NativeEngine,
        document_id: &str,
        filename: &str,
        pdf_bytes: &[u8],
        language: &str,
        options: PreprocessOptions,
        cancel_flag: &AtomicBool,
        mut progress_cb: F,
    ) -> Result<OcrResult, String>
    where
        F: FnMut(u32, u32, &str), // (current_page, total_pages, stage_message)
    {
        if pdf_bytes.is_empty() {
            return Err("Cannot process empty PDF document".to_string());
        }

        let start_time = Instant::now();

        let mut warnings = Vec::new();

        // 1. Attempt to load PDF structure with lopdf for fast digital text path
        let lopdf_res = Document::load_mem(pdf_bytes);
        let (lopdf_doc, page_numbers, total_pages) = match lopdf_res {
            Ok(doc) => {
                let p_nums: Vec<u32> = doc.get_pages().keys().copied().collect();
                let count = p_nums.len() as u32;
                if count > 0 {
                    (Some(doc), p_nums, count)
                } else {
                    let cg_count = NativeEngine::pdf_page_count(pdf_bytes).unwrap_or(0);
                    let fallback_nums: Vec<u32> = (1..=cg_count).collect();
                    (None, fallback_nums, cg_count)
                }
            }
            Err(e) => {
                // Robust fallback to native CoreGraphics engine
                match NativeEngine::pdf_page_count(pdf_bytes) {
                    Ok(count) if count > 0 => {
                        warnings.push(format!(
                            "PDF structure parser fallback (CoreGraphics): {}",
                            e
                        ));
                        let fallback_nums: Vec<u32> = (1..=count).collect();
                        (None, fallback_nums, count)
                    }
                    _ => {
                        return Err(format!("Failed to parse PDF document structure: {}", e));
                    }
                }
            }
        };

        if total_pages == 0 {
            return Err("PDF contains zero pages".to_string());
        }

        let mut pages = Vec::with_capacity(total_pages as usize);
        // Pages that need OCR but cannot be rasterised because this platform has no PDF engine
        // (Windows). Never happens on macOS.
        let mut unsupported_pages: u32 = 0;
        let mut readable_pages: u32 = 0;

        for (idx, &page_id) in page_numbers.iter().enumerate() {
            let page_idx = (idx + 1) as u32;

            // Check cancellation token
            if cancel_flag.load(Ordering::SeqCst) {
                return Err("PDF processing cancelled by user".to_string());
            }

            progress_cb(page_idx, total_pages, "Analyzing page content type");

            // Attempt digital text extraction if lopdf loaded successfully
            let digital_text = if let Some(ref doc) = lopdf_doc {
                doc.extract_text(&[page_id]).unwrap_or_default()
            } else {
                String::new()
            };

            if is_digital_text_viable(&digital_text) {
                // Clean digital text page: direct extraction preserves original Unicode and dimensions
                progress_cb(page_idx, total_pages, "Extracted digital text layer");
                let clean_text = normalize_digital_text(&digital_text);

                // Generate synthetic line blocks
                let mut blocks = Vec::new();
                for (line_idx, line) in clean_text.lines().enumerate() {
                    let line_clean = line.trim();
                    if !line_clean.is_empty() {
                        blocks.push(OcrBlock {
                            bbox: BoundingBox {
                                x: 50,
                                y: (50 + line_idx * 25) as i32,
                                width: (line_clean.len() * 8) as i32,
                                height: 20,
                            },
                            confidence: 100.0,
                            text: line_clean.to_string(),
                        });
                    }
                }

                readable_pages += 1;
                pages.push(OcrPage {
                    page: page_idx,
                    text: clean_text,
                    blocks,
                    confidence: 100.0,
                });
            } else {
                // Scanned image or fragmented math page: render to 300 DPI raster and run OCR
                progress_cb(page_idx, total_pages, "Rendering page at 300 DPI");

                let rendered_png = match NativeEngine::render_pdf_page(pdf_bytes, page_idx, 300) {
                    Ok(png) => png,
                    Err(e) if e == PDF_RASTER_UNSUPPORTED => {
                        unsupported_pages += 1;
                        pages.push(OcrPage {
                            page: page_idx,
                            text: format!("[Page {} is a scan and was not read: OCR for PDF pages is not available on this platform yet]", page_idx),
                            blocks: Vec::new(),
                            confidence: 0.0,
                        });
                        continue;
                    }
                    Err(e) => {
                        warnings.push(format!("Page {}: rasterization failed: {}", page_idx, e));
                        pages.push(OcrPage {
                            page: page_idx,
                            text: format!("[Page {} rasterization error: {}]", page_idx, e),
                            blocks: Vec::new(),
                            confidence: 0.0,
                        });
                        continue;
                    }
                };

                progress_cb(
                    page_idx,
                    total_pages,
                    "Running optical character recognition",
                );

                let page_doc_id = format!("{}_p{}", document_id, page_idx);
                let page_filename = format!("{}_page_{}.png", filename, page_idx);

                let page_ocr = match engine.run_ocr(
                    &page_doc_id,
                    &page_filename,
                    &rendered_png,
                    language,
                    options,
                ) {
                    Ok(ocr) => ocr,
                    Err(e) => {
                        warnings.push(format!("Page {}: OCR engine failed: {}", page_idx, e));
                        pages.push(OcrPage {
                            page: page_idx,
                            text: format!("[Page {} OCR failed: {}]", page_idx, e),
                            blocks: Vec::new(),
                            confidence: 0.0,
                        });
                        continue;
                    }
                };

                if let Some(mut first_page) = page_ocr.pages.into_iter().next() {
                    first_page.page = page_idx;
                    readable_pages += 1;
                    pages.push(first_page);
                } else {
                    warnings.push(format!("Page {}: No text detected by OCR engine", page_idx));
                    pages.push(OcrPage {
                        page: page_idx,
                        text: String::new(),
                        blocks: Vec::new(),
                        confidence: 0.0,
                    });
                }
            }
        }

        if unsupported_pages > 0 {
            if readable_pages == 0 {
                // Nothing could be read at all: report the reason instead of a "successful"
                // result made only of placeholders.
                return Err(PDF_RASTER_UNSUPPORTED.to_string());
            }
            warnings.push(format!(
                "{} scanned page(s) were skipped. {}",
                unsupported_pages, PDF_RASTER_UNSUPPORTED
            ));
        }

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(OcrResult {
            document_id: document_id.to_string(),
            filename: filename.to_string(),
            language: language.to_string(),
            pages,
            processing_time_ms: elapsed_ms,
            warnings,
        })
    }
}
