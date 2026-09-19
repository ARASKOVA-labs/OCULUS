use crate::models::{BoundingBox, OcrBlock, OcrPage, OcrResult, PreprocessOptions};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

#[repr(C)]
struct OculusEngineOpaque {
    _unused: [u8; 0],
}

#[repr(C)]
struct OculusPreprocessOptionsC {
    grayscale: i32,
    denoise: i32,
    deskew: i32,
    threshold: i32,
    orientation_detect: i32,
    contrast_norm: i32,
}

impl From<PreprocessOptions> for OculusPreprocessOptionsC {
    fn from(opts: PreprocessOptions) -> Self {
        Self {
            grayscale: if opts.grayscale { 1 } else { 0 },
            denoise: if opts.denoise { 1 } else { 0 },
            deskew: if opts.deskew { 1 } else { 0 },
            threshold: if opts.threshold { 1 } else { 0 },
            orientation_detect: if opts.orientation_detect { 1 } else { 0 },
            contrast_norm: if opts.contrast_norm { 1 } else { 0 },
        }
    }
}

#[repr(C)]
struct OculusBoundingBoxC {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[repr(C)]
struct OculusBlockC {
    bbox: OculusBoundingBoxC,
    confidence: f32,
    text: *const c_char,
}

#[repr(C)]
struct OculusPageC {
    page_number: i32,
    confidence: f32,
    full_text: *const c_char,
    block_count: usize,
    blocks: *const OculusBlockC,
}

#[repr(C)]
struct OculusResultC {
    success: i32,
    error_message: *const c_char,
    page_count: usize,
    pages: *const OculusPageC,
    processing_time_ms: u64,
}

extern "C" {
    fn oculus_engine_create(tessdata_prefix: *const c_char) -> *mut OculusEngineOpaque;
    fn oculus_engine_destroy(engine: *mut OculusEngineOpaque);
    fn oculus_engine_preprocess(
        in_data: *const u8,
        in_size: usize,
        options: OculusPreprocessOptionsC,
        out_data: *mut *mut u8,
        out_size: *mut usize,
    ) -> c_int;
    fn oculus_engine_ocr(
        engine: *mut OculusEngineOpaque,
        image_data: *const u8,
        image_size: usize,
        language: *const c_char,
        options: OculusPreprocessOptionsC,
        out_result: *mut *mut OculusResultC,
    ) -> c_int;
    fn oculus_engine_pdf_page_count(pdf_data: *const u8, pdf_size: usize) -> c_int;
    fn oculus_engine_render_pdf_page(
        pdf_data: *const u8,
        pdf_size: usize,
        page_number: c_int,
        dpi: c_int,
        out_png_data: *mut *mut u8,
        out_png_size: *mut usize,
    ) -> c_int;
    fn oculus_engine_create_searchable_pdf(
        engine: *mut OculusEngineOpaque,
        image_data: *const u8,
        image_size: usize,
        language: *const c_char,
        output_pdf_base: *const c_char,
    ) -> c_int;
    fn oculus_free_result(result: *mut OculusResultC);
    fn oculus_free_buffer(buffer: *mut u8);
}

/// Safe RAII wrapper around the C++ native OCR engine.
pub struct NativeEngine {
    raw: *mut OculusEngineOpaque,
}

// Safety: The C++ engine implementation is thread-safe for recognition once initialized.
unsafe impl Send for NativeEngine {}
unsafe impl Sync for NativeEngine {}

impl Drop for NativeEngine {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // Safety: raw pointer was created by oculus_engine_create and is valid until dropped.
            unsafe {
                oculus_engine_destroy(self.raw);
            }
            self.raw = std::ptr::null_mut();
        }
    }
}

impl NativeEngine {
    /// Initialize native engine with an optional path to tessdata.
    pub fn new(tessdata_prefix: Option<&str>) -> Result<Arc<Self>, String> {
        // Resolve tessdata directory through discovery cascade
        let resolved_path = if let Some(p) = tessdata_prefix {
            Some(p.to_string())
        } else if let Ok(env_path) = std::env::var("TESSDATA_PREFIX") {
            Some(env_path)
        } else {
            // Check relative to current executable (packaged app / bundle)
            let mut candidates = Vec::new();
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    candidates.push(exe_dir.join("tessdata"));
                    candidates.push(exe_dir.join("../Resources/tessdata"));
                    candidates.push(exe_dir.join("../Resources"));
                }
            }
            // Standard OS distribution locations
            candidates.push(std::path::PathBuf::from("/opt/homebrew/share/tessdata"));
            candidates.push(std::path::PathBuf::from("/usr/local/share/tessdata"));
            candidates.push(std::path::PathBuf::from("/usr/share/tesseract-ocr/5/tessdata"));
            candidates.push(std::path::PathBuf::from("/usr/share/tessdata"));

            candidates
                .into_iter()
                .find(|p| p.exists())
                .map(|p| p.to_string_lossy().into_owned())
        };

        let prefix_cstr = match resolved_path {
            Some(ref p) => Some(CString::new(p.as_str()).map_err(|e| format!("Invalid path: {}", e))?),
            None => None,
        };

        let ptr = prefix_cstr
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());

        // Safety: Calls extern C constructor which returns a valid pointer or null.
        let raw = unsafe { oculus_engine_create(ptr) };
        if raw.is_null() {
            return Err("Failed to allocate native OCR engine".to_string());
        }

        Ok(Arc::new(Self { raw }))
    }

    /// Preprocess image buffer using Leptonica / native C++ algorithms.
    pub fn preprocess_image(
        in_bytes: &[u8],
        options: PreprocessOptions,
    ) -> Result<Vec<u8>, String> {
        if in_bytes.is_empty() {
            return Err("Empty input buffer".to_string());
        }

        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let c_opts = OculusPreprocessOptionsC::from(options);

        // Safety: in_bytes is a valid continuous buffer. out_ptr and out_len are output pointers.
        let rc = unsafe {
            oculus_engine_preprocess(
                in_bytes.as_ptr(),
                in_bytes.len(),
                c_opts,
                &mut out_ptr,
                &mut out_len,
            )
        };

        if rc != 0 || out_ptr.is_null() {
            return Err(format!("Native preprocessing failed with code {}", rc));
        }

        // Safety: C++ allocated out_ptr with out_len bytes. We copy to safe Vec and free the C buffer.
        let result_vec = unsafe {
            let slice = std::slice::from_raw_parts(out_ptr, out_len);
            let v = slice.to_vec();
            oculus_free_buffer(out_ptr);
            v
        };

        Ok(result_vec)
    }

    /// Execute OCR recognition on the provided image buffer.
    pub fn run_ocr(
        &self,
        document_id: &str,
        filename: &str,
        image_bytes: &[u8],
        language: &str,
        options: PreprocessOptions,
    ) -> Result<OcrResult, String> {
        if image_bytes.is_empty() {
            return Err("Cannot perform OCR on empty image buffer".to_string());
        }

        let lang_cstr = CString::new(language).map_err(|e| format!("Invalid language string: {}", e))?;
        let c_opts = OculusPreprocessOptionsC::from(options);
        let mut result_ptr: *mut OculusResultC = std::ptr::null_mut();

        // Safety: Calls C++ OCR engine with valid pointers.
        let rc = unsafe {
            oculus_engine_ocr(
                self.raw,
                image_bytes.as_ptr(),
                image_bytes.len(),
                lang_cstr.as_ptr(),
                c_opts,
                &mut result_ptr,
            )
        };

        if rc != 0 || result_ptr.is_null() {
            return Err(format!("Native OCR engine failed with return code {}", rc));
        }

        // Safety: result_ptr is non-null and valid. We copy all data into Rust owned structs and call oculus_free_result.
        let ocr_result = unsafe {
            let res = &*result_ptr;
            if res.success == 0 {
                let err_msg = if !res.error_message.is_null() {
                    CStr::from_ptr(res.error_message)
                        .to_string_lossy()
                        .into_owned()
                } else {
                    "Unknown native OCR error".to_string()
                };
                oculus_free_result(result_ptr);
                return Err(err_msg);
            }

            let mut pages = Vec::with_capacity(res.page_count);
            if res.page_count > 0 && !res.pages.is_null() {
                let page_slice = std::slice::from_raw_parts(res.pages, res.page_count);
                for p in page_slice {
                    let page_text = if !p.full_text.is_null() {
                        CStr::from_ptr(p.full_text)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        String::new()
                    };

                    let mut blocks = Vec::with_capacity(p.block_count);
                    if p.block_count > 0 && !p.blocks.is_null() {
                        let block_slice = std::slice::from_raw_parts(p.blocks, p.block_count);
                        for b in block_slice {
                            let text = if !b.text.is_null() {
                                CStr::from_ptr(b.text)
                                    .to_string_lossy()
                                    .into_owned()
                            } else {
                                String::new()
                            };

                            blocks.push(OcrBlock {
                                bbox: BoundingBox {
                                    x: b.bbox.x,
                                    y: b.bbox.y,
                                    width: b.bbox.width,
                                    height: b.bbox.height,
                                },
                                confidence: b.confidence,
                                text,
                            });
                        }
                    }

                    pages.push(OcrPage {
                        page: p.page_number as u32,
                        text: page_text,
                        blocks,
                        confidence: p.confidence,
                    });
                }
            }

            let final_result = OcrResult {
                document_id: document_id.to_string(),
                filename: filename.to_string(),
                language: language.to_string(),
                pages,
                processing_time_ms: res.processing_time_ms,
                warnings: Vec::new(),
            };

            oculus_free_result(result_ptr);
            final_result
        };

        Ok(ocr_result)
    }

    /// Query the number of pages in a PDF document buffer.
    pub fn pdf_page_count(pdf_bytes: &[u8]) -> Result<u32, String> {
        if pdf_bytes.is_empty() {
            return Err("Empty PDF buffer".to_string());
        }
        let count = unsafe {
            oculus_engine_pdf_page_count(pdf_bytes.as_ptr(), pdf_bytes.len())
        };
        if count < 0 {
            Err("Failed to parse PDF pages".to_string())
        } else {
            Ok(count as u32)
        }
    }

    /// Render a single PDF page to a PNG bitmap buffer at target DPI.
    pub fn render_pdf_page(pdf_bytes: &[u8], page_number: u32, dpi: u32) -> Result<Vec<u8>, String> {
        if pdf_bytes.is_empty() {
            return Err("Empty PDF buffer".to_string());
        }
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;

        let rc = unsafe {
            oculus_engine_render_pdf_page(
                pdf_bytes.as_ptr(),
                pdf_bytes.len(),
                page_number as i32,
                dpi as i32,
                &mut out_ptr,
                &mut out_len,
            )
        };

        if rc != 0 || out_ptr.is_null() {
            return Err(format!("PDF page rendering failed with code {}", rc));
        }

        let png_bytes = unsafe {
            let slice = std::slice::from_raw_parts(out_ptr, out_len);
            let v = slice.to_vec();
            oculus_free_buffer(out_ptr);
            v
        };

        Ok(png_bytes)
    }

    /// Generate a real searchable PDF with invisible text layer from image data.
    pub fn create_searchable_pdf(
        &self,
        image_bytes: &[u8],
        language: &str,
        output_pdf_base: &str,
    ) -> Result<(), String> {
        if image_bytes.is_empty() {
            return Err("Cannot generate searchable PDF from empty image buffer".to_string());
        }

        let lang_cstr = CString::new(language).map_err(|e| format!("Invalid language string: {}", e))?;
        let out_cstr = CString::new(output_pdf_base).map_err(|e| format!("Invalid output base string: {}", e))?;

        let rc = unsafe {
            oculus_engine_create_searchable_pdf(
                self.raw,
                image_bytes.as_ptr(),
                image_bytes.len(),
                lang_cstr.as_ptr(),
                out_cstr.as_ptr(),
            )
        };

        if rc != 0 {
            return Err(format!("Searchable PDF generation failed with code {}", rc));
        }

        Ok(())
    }
}
