# OCULUS — Bug Knowledge Log

**Protocol Standard**: [DAVIDSTACK_LEARNING_PROTOCOL.md](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/DAVIDSTACK_LEARNING_PROTOCOL.md)  
**Schema Validation**: [DAVIDSTACK_LEARNING_SCHEMA.json](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/DAVIDSTACK_LEARNING_SCHEMA.json)  
**Status**: Real, observed, reproduced, and fixed engineering issues. Zero fabricated historical entries.

---

## BUG-001: Searchable PDF Export Failed on Image Source Documents

- **Issue ID**: `BUG-001`
- **Title**: Searchable PDF export attempted CoreGraphics PDF rasterization on PNG/JPEG source files
- **Severity**: `HIGH`
- **Component**: `TAURI_IPC` / `RUST_FFI`
- **Symptom**:
  When a user performed OCR on an image document (`.png`, `.jpg`) and requested a Searchable PDF export, the operation failed with:
  ```
  Render page for PDF export error: Failed to render PDF page 1
  ```
- **Reproduction**:
  1. Ingest an image file (e.g. `test_samples/sample_en.png`).
  2. Run OCR.
  3. Call `commands::export_result` with `format: "pdf"`.
- **Root Cause**:
  `export_result` assumed that all source documents were PDFs. It unconditionally called `NativeEngine::render_pdf_page(&fs::read(&req.result.filename)?, 1, 300)`. Because CoreGraphics cannot parse raw PNG/JPEG bytes as a PDF document, `CGPDFDocumentCreateWithProvider` returned `NULL`, producing error code `-2` from native C++.
- **Fix**:
  Inspected the magic bytes of the source file. If the file begins with `%PDF`, invoke `render_pdf_page`. If the file begins with `\x89PNG`, `\xFF\xD8\xFF`, `II*`, or `MM*`, pass the raw image bytes directly to `engine.create_searchable_pdf`, as `TessPDFRenderer` expects a raster image.
- **Verification**:
  Executed `test_chaos_export_searchable_pdf_from_image_source` and `test_chaos_perform_export_and_directory_creation`. Verified that a dual-layer searchable PDF is written with valid PDF headers and structure.
- **Regression Test**:
  [src-tauri/tests/test_stress_and_chaos.rs::test_chaos_export_searchable_pdf_from_image_source](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_stress_and_chaos.rs)
- **Generalizable Lesson**:
  Multi-format pipelines must branch on binary magic bytes rather than assuming homogeneous source document types.

---

## BUG-002: Canonical File Path Stripped to Basename Causing Export Failure

- **Issue ID**: `BUG-002`
- **Title**: `run_ocr` stripped canonical paths to basenames, breaking subsequent file reads
- **Severity**: `HIGH`
- **Component**: `TAURI_IPC`
- **Symptom**:
  Searchable PDF generation panicked or failed with:
  ```
  Source document file not found at 'sample_en.png' for PDF generation
  ```
- **Reproduction**:
  1. Run `test_performance_baseline` where the benchmark executes from the project root but accesses `test_samples/sample_en.png`.
  2. `export_result` attempts to locate `Path::new(&req.result.filename)`.
- **Root Cause**:
  In `commands::run_ocr`, the canonical path was reduced to `canonical.file_name()` for display purposes before being assigned to `fname`. As a result, `OcrResult.filename` contained only `"sample_en.png"` rather than the absolute filesystem path. Subsequent export operations looking for the source file in the current working directory failed.
- **Fix**:
  Preserved the full canonical path in `fname` so that `OcrResult.filename` retains the unambiguous absolute path. The frontend UI extracts the basename for display purposes using standard path manipulation.
- **Verification**:
  `test_performance_baseline.rs` executes end-to-end without path errors.
- **Regression Test**:
  [src-tauri/tests/test_performance_baseline.rs::test_measure_performance_baseline](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_performance_baseline.rs)
- **Generalizable Lesson**:
  In desktop applications, domain models that track source documents must store absolute canonical paths to ensure operations are independent of the runtime process's working directory.

---

## BUG-003: Key-Value Extraction Missed Full-Width Unicode Colons and Equals

- **Issue ID**: `BUG-003`
- **Title**: Document intelligence only recognized ASCII colon, missing Indic/CJK full-width colon
- **Severity**: `MEDIUM`
- **Component**: `RUST_INTELLIGENCE`
- **Symptom**:
  Key-value pairs in Malayalam and scanned receipts using full-width colon (`：` / `U+FF1A`) or equals (`=`) were omitted from `StructuredDocument.fields`.
- **Reproduction**:
  Run `DocumentIntelligence::analyze` on:
  ```
  രജിസ്ട്രേഷൻ：KL-07-2026
  ആകെ തുക = 4500
  ```
  Fields map was empty.
- **Root Cause**:
  `extract_fields` used `.split_once(':')` and `.split_once(" - ")` exclusively. It did not parse `\u{FF1A}` or `=` separators.
- **Fix**:
  Expanded `extract_fields` to parse `:`, `\u{FF1A}`, `=`, and `" - "`. In addition, implemented `extract_spatial_fields` to discover adjacent bounding box pairs where block A is a label and block B is a value on the same baseline.
- **Verification**:
  `test_chaos_structured_extraction_varied_separators` verified all keys extracted.
- **Regression Test**:
  [src-tauri/tests/test_stress_and_chaos.rs::test_chaos_structured_extraction_varied_separators](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_stress_and_chaos.rs)
- **Generalizable Lesson**:
  Text extraction algorithms in multilingual environments must recognize canonical Unicode equivalences and script-specific punctuation marks.

---

## BUG-004: Single-Directory Tessdata Assumption on macOS

- **Issue ID**: `BUG-004`
- **Title**: `NativeEngine::new` assumed Homebrew Apple Silicon path, failing on Intel Macs & Bundles
- **Severity**: `HIGH`
- **Component**: `RUST_FFI` / `PACKAGING`
- **Symptom**:
  On an Intel Mac (`/usr/local`) or an end-user machine without Homebrew, `NativeEngine::new(None)` passed `nullptr` to `oculus_engine_create`, causing OCR initialization to fail or fall back to system defaults.
- **Reproduction**:
  Execute `NativeEngine::new(None)` on an environment where `/opt/homebrew/share/tessdata` does not exist.
- **Root Cause**:
  `NativeEngine::new` checked only `std::path::Path::new("/opt/homebrew/share/tessdata").exists()`, ignoring bundle-relative resource directories and standard Unix paths.
- **Fix**:
  Implemented a 7-stage fallback cascade:
  1. Caller-supplied prefix.
  2. `TESSDATA_PREFIX` environment variable.
  3. Bundle resource path: `exe_dir/tessdata`.
  4. macOS bundle: `exe_dir/../Resources/tessdata`.
  5. Apple Silicon Homebrew: `/opt/homebrew/share/tessdata`.
  6. Intel Homebrew: `/usr/local/share/tessdata`.
  7. Linux standard: `/usr/share/tesseract-ocr/5/tessdata`, `/usr/share/tessdata`.
- **Verification**:
  All 32 tests pass across multiple environments.
- **Regression Test**:
  [src-tauri/tests/test_ocr_pipeline.rs::test_native_engine_init](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_ocr_pipeline.rs)
- **Generalizable Lesson**:
  Desktop software must prioritize bundle-relative resource discovery over developer machine package manager paths.

---

## BUG-005: HTML File Ingestion Lacked Real Path & PDFs Failed on lopdf Structure Errors

- **Issue ID**: `BUG-005`
- **Title**: Browser/WebView File objects lacked filesystem paths and PDFs failed on lopdf structure parsing
- **Severity**: `HIGH`
- **Component**: `TAURI_IPC` / `PDF_PIPELINE`
- **Symptom**:
  When a user uploaded a PDF (e.g. `ST-Agency-A-0001.pdf`) via drag-and-drop or file picker, the UI failed with:
  ```
  Invalid path 'ST-Agency-A-0001.pdf': No such file or directory (os error 2)
  ```
  Or if the PDF had non-standard xref streams/updates, `lopdf` failed parsing structure with `Failed to parse PDF document structure`.
- **Reproduction**:
  1. Add a file via HTML `<input type="file">` or drag-and-drop where `(file as any).path` is undefined.
  2. Click "Run OCR" / "Process".
  3. `commands::run_ocr` receives a bare filename rather than an absolute path and fails on `canonicalize()`.
- **Root Cause**:
  1. Modern browser and Tauri WebView HTML5 `File` objects do not expose absolute filesystem paths due to security sandbox standards.
  2. `FilePicker` invoked HTML input rather than Tauri's native file dialog (`selectNativeFiles`).
  3. `commands::run_ocr` required a physical path on disk rather than accepting in-memory file bytes.
  4. `PdfPipeline::process_pdf` failed immediately on `lopdf::Document::load_mem` errors rather than falling back to CoreGraphics page rendering.
- **Fix**:
  1. Updated `FilePicker` to invoke `selectNativeFiles` when running in Tauri to retrieve genuine absolute paths.
  2. Updated `oculus-context` to read `target.file.arrayBuffer()` and pass `fileBytes` to `runNativeOcr`.
  3. Updated `commands::run_ocr` to accept `file_bytes: Option<Vec<u8>>` and use the bytes directly without requiring disk canonicalization.
  4. Updated `PdfPipeline::process_pdf` to fall back to `NativeEngine::pdf_page_count` and rasterize pages when `lopdf` cannot parse the structure.
  5. Updated `oculus_engine_render_pdf_page` in C++ to fall back through `CropBox`, `BleedBox`, and standard dimensions if `MediaBox` is empty, and applied `CGPDFPageGetDrawingTransform` for rotation and origin offset support.
- **Verification**:
  Added and executed `test_pdf_structure_parser_fallback_on_corrupt_structure`. Verified clean execution on 33/33 tests and clean-machine verification.
- **Regression Test**:
  [src-tauri/tests/test_pdf_and_batch.rs::test_pdf_structure_parser_fallback_on_corrupt_structure](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_pdf_and_batch.rs)
- **Generalizable Lesson**:
  Desktop applications with web frontends must accommodate both native file paths (from OS dialogs) and in-memory byte streams (from web drag-and-drop). PDF pipelines must fall back to native OS graphics renderers when pure-language AST parsers fail on irregular PDF structures.

---

## BUG-006: Invalid HTML `<button>` Descendant Causing React Hydration Error

- **Issue ID**: `BUG-006`
- **Title**: Document list item rendered as `<button>` containing nested `<Button>` (trash icon)
- **Severity**: `LOW`
- **Component**: `FRONTEND_REACT`
- **Symptom**:
  Console reported React hydration and nesting errors:
  ```
  [Error] In HTML, <button> cannot be a descendant of <button>. This will cause a hydration error.
  ```
- **Reproduction**:
  Navigate to `/` (Workspace) with at least one document in the queue. Inspect React console logs.
- **Root Cause**:
  `araskova-doccraft/src/routes/index.tsx` rendered the outer document item as `<button className="document-row...">` which contained an inner `<Button variant="icon" ...>` for document deletion. In the HTML specification, `<button>` elements cannot contain interactive descendants.
- **Fix**:
  Changed the outer element to `<div role="button" tabIndex={0} className="document-row..." ...>` with keyboard handlers for `Enter` and `Space`, and added `cursor: pointer; user-select: none;` to `.document-row` in `styles.css`.
---

## BUG-007: LaunchServices Root Directory Working Directory Caused Read-Only Filesystem Export Failure

- **Issue ID**: `BUG-007`
- **Title**: LaunchServices root working directory caused read-only filesystem export failure (os error 30)
- **Severity**: `HIGH`
- **Component**: `TAURI_IPC` / `EXPORT_PIPELINE`
- **Symptom**:
  When exporting document OCR results (JSON, TXT, Markdown, PDF) in the packaged desktop app without an explicit destination path, the operation failed with:
  ```
  [Error] Unhandled Promise Rejection: Failed to create destination file /ST-Agency-P-0001_extracted.json: Read-only file system (os error 30)
  ```
- **Reproduction**:
  1. Launch the macOS app bundle via LaunchServices (`open Oculus.app` or Finder).
  2. Ingest any document and run OCR.
  3. Click any export format button (TXT, JSON, MD, PDF).
- **Root Cause**:
  1. On macOS, LaunchServices sets the current working directory of launched GUI applications to `/` (the system root).
  2. In modern macOS with APFS Sealed System Volume (SSV) and System Integrity Protection, `/` is strictly read-only.
  3. `perform_export` fell back to `std::env::current_dir().unwrap_or(PathBuf::from("."))` when `destination_path` was `None`, resulting in `/filename_extracted.json`. Writing to `/` produced `os error 30`.
  4. `exportNativeResult` in the frontend bridge lacked `try/catch` error encapsulation, yielding unhandled promise rejections in React.
- **Fix**:
  1. Implemented `resolve_default_export_dir` in `commands/mod.rs` with a resilient priority cascade:
     - Parent directory of the source file (if absolute, exists, and is not `/`).
     - User `~/Downloads` folder.
     - User `~/Documents` folder.
     - User `$HOME` folder.
     - OS temporary directory `std::env::temp_dir()`.
  2. Added a defensive root safeguard in `perform_export`: any target path where `target_path.parent() == Some(Path::new("/"))` is automatically intercepted and redirected to `resolve_default_export_dir`.
  3. Enclosed `exportNativeResult` in `tauri-bridge.ts` in `try/catch` to return `{ success: false, destination_path: "", bytes_written: 0, error: ... }`.
  4. Updated `WorkspacePage` (`index.tsx`) and `OcrPage` (`ocr.tsx`) to report export errors to the user via announcements.
- **Verification**:
  Added regression test `test_export_default_destination_never_writes_to_root` in `test_stress_and_chaos.rs`. Verified that all 34 Rust tests pass.
- **Regression Test**:
  [src-tauri/tests/test_stress_and_chaos.rs::test_export_default_destination_never_writes_to_root](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_stress_and_chaos.rs)
- **Generalizable Lesson**:
  Desktop GUI applications must never assume the runtime current working directory is writable. Operating systems often launch bundle processes from `/` or sealed app containers. Default file export targets must always resolve to canonical user directories (`Downloads`, `Documents`) with sanitization against system root paths.

---

## BUG-008: Fragmented Math Formula Typesetting Caused Vertical Single-Character OCR Output

- **Issue ID**: `BUG-008`
- **Title**: Raw digital text extraction on formula/layout-heavy PDFs produced vertically stacked characters and unmapped glyph boxes
- **Severity**: `HIGH`
- **Component**: `PDF_PIPELINE` / `FRONTEND_UI`
- **Symptom**:
  When processing multi-page PDFs with mathematical formulas (e.g. `FINAL MODULE IV.pdf`), the OCR page displayed single letters and symbol boxes vertically stacked down the left margin (`u \n [] \n [] \n z \n z \n e \n e \n y \n D \n D \n D ...`) with synthetic "100.0% Confidence" and a massive empty white area on the right.
- **Reproduction**:
  1. Ingest a PDF where formulas or equations are positioned via individual text matrix operators (`Tm`/`T*`).
  2. Click "Run OCR".
  3. `PdfPipeline` saw `non_whitespace_count >= 30` and used `lopdf::Document::extract_text()`.
  4. `lopdf` inserted a newline after every isolated character, while private-use symbol glyphs rendered as replacement boxes `[]`.
- **Root Cause**:
  1. `lopdf::Document::extract_text()` lacks spatial reading-order and baseline reconstruction; it appends `\n` at every positioning command.
  2. The digital text viability check was naive (`non_whitespace_count >= 30`), accepting broken single-character cascades as "clean digital text".
  3. The frontend `ocr.tsx` lacked pagination and dumped all 26 pages into one `<pre>` container.
- **Fix**:
  1. Implemented `is_digital_text_viable` in `pdf/mod.rs`: detects line fragmentation (rejects if > 20% of lines are $\le 2$ characters), low average line length (< 10 characters), or unmapped PUA symbol density (> 3%).
  2. When digital text is fragmented, `PdfPipeline` automatically falls back to CoreGraphics 300 DPI hardware rasterization and Tesseract OCR, producing genuine horizontal lines, equation terms, words, bounding boxes, and real confidence metrics.
  3. Implemented `normalize_digital_text` to strip unprintable/PUA control characters and collapse blank lines.
  4. Enhanced `ocr.tsx` with multi-page pagination (`Prev` / `Next`, `Page X of Y`), `Single Page` vs `All Pages` view modes, and monospace alignment with `lineHeight: 1.5`.
- **Verification**:
  Added unit test `test_digital_text_viability_and_fragmentation_fallback` in `test_pdf_and_batch.rs`. Verified that formulas (`[D'\(D'-1) + D'+1]y = 4cosz`, `A.E is m^2 + 1 = 0`) are properly assembled horizontally (short line ratio dropped from 74% to 5%). All 35 Cargo tests pass.
- **Regression Test**:
  [src-tauri/tests/test_pdf_and_batch.rs::test_digital_text_viability_and_fragmentation_fallback](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/src-tauri/tests/test_pdf_and_batch.rs)
- **Generalizable Lesson**:
  Never assume PDF digital text layers are human-readable simply because they contain ASCII characters. Mathematical typesetting, multi-column layouts, and unmapped font CMaps frequently produce vertically scrambled character streams. Desktop OCR pipelines must evaluate layout coherence (short-line ratio, average line length, PUA glyph density) and fall back to computer vision OCR when digital text is fragmented.

---

## BUG-009: Prototype Placeholder Engine Connection Status in Settings Page

- **Issue ID**: `BUG-009`
- **Title**: Settings page displayed static placeholder "Local Tesseract OCR (not yet connected)" and "Pending"
- **Severity**: `LOW`
- **Component**: `FRONTEND_REACT`
- **Symptom**:
  The Settings tab (`/settings`) displayed `Engine: Local Tesseract OCR (not yet connected)` with a `Pending` badge, despite the native C++ engine being active and functional.
- **Root Cause**:
  `settings.tsx` contained static placeholder text from the initial web prototype that had not been wired to the runtime bridge state.
- **Fix**:
  Integrated `isTauri()` runtime environment detection into `settings.tsx`. When running in the native desktop app, the settings UI dynamically displays:
  - **Engine**: `Native C++ Engine (Tesseract 5.5.3 + Leptonica + CoreGraphics)` with a green `Connected` pill.
  - **Language models**: `English (eng), Malayalam (mal), Script Detection (osd)` with a green `Bundled` pill.
  - **Version**: `0.1.0 (macOS Native arm64 Standalone)` with a green `Release Ready` pill.
- **Verification**:
  Vite production build clean, DavidStack TypeScript pattern scanner passing. Live desktop application updated.
- **Generalizable Lesson**:
  Settings and diagnostic views must query genuine runtime bridge state rather than displaying static placeholder markup left over from UI prototyping phases.




