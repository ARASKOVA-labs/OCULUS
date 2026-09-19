# OCULUS Implementation Plan

## Phase 0 — Repository inspection
1. Inspect all project files.
2. Locate and read DavidStack if present.
3. Treat DavidStack as an engineering/product-quality rule layer.
4. Reconcile DavidStack with this specification.
5. Create an implementation checklist before coding.

## Phase 1 — Skeleton
- Create Tauri application.
- Create React/TypeScript frontend.
- Create Rust application layer.
- Create C++ library target.
- Establish Rust↔C++ FFI.
- Establish typed frontend↔Rust IPC.
- Add CI and formatting/linting.

## Phase 2 — File pipeline
- File picker/drop zone.
- File validation.
- Document inspection.
- Temporary workspace management.
- Job queue.

## Phase 3 — Image pipeline
- OpenCV integration.
- Rotation.
- Grayscale.
- Denoise.
- Threshold.
- Deskew.
- Preview preprocessing.

## Phase 4 — OCR
- Integrate initial OCR backend.
- Language selection.
- Result model.
- Confidence/warnings.
- Multi-page processing.

## Phase 5 — PDF
- Detect text/scanned pages.
- OCR scanned pages.
- Searchable PDF output.
- Batch PDF support.

## Phase 6 — Export
- TXT
- Markdown
- JSON
- JSONL

## Phase 7 — UX hardening
- cancellation
- recovery
- error states
- accessibility
- keyboard shortcuts
- performance profiling

## Phase 8 — Malayalam readiness
- strict UTF-8 pipeline
- Unicode normalization strategy
- Malayalam language configuration
- regression corpus
- dataset-oriented JSONL export

## Phase 9 — Release
- Windows build
- macOS build
- Linux build
- signed installers where available
- documentation
- reproducible build instructions

## Engineering rule
Do not jump to Phase 9 until critical Phase 2–8 tests pass.
