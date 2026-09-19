# OCULUS IPC / API Specification

The frontend communicates only with the Rust/Tauri layer.

## Commands

### `select_files`
Returns validated file descriptors.

### `inspect_document`
Input:
- file path/handle
Returns:
- document type
- page count
- dimensions
- text presence
- warnings

### `preprocess`
Input:
- document/page
- preprocessing options
Returns:
- processed artifact handle
- warnings

### `ocr_document`
Input:
- document handle
- language
- preprocessing options
- OCR configuration
Returns:
- job ID

### `get_job`
Input:
- job ID
Returns:
- state
- progress
- current file/page
- errors
- outputs

### `cancel_job`
Input:
- job ID
Returns:
- cancellation status

### `export_result`
Input:
- result ID
- format
- destination
Returns:
- output path
- metadata

### `create_searchable_pdf`
Input:
- source document
- OCR result
- destination
Returns:
- output path

## Event model
Rust emits:
- `job_started`
- `job_progress`
- `page_completed`
- `file_completed`
- `job_warning`
- `job_error`
- `job_completed`

All payloads must be versioned and typed.

## Error model
Use machine-readable error codes plus user-facing messages.

Example categories:
- INVALID_FILE
- UNSUPPORTED_FORMAT
- OCR_ENGINE_ERROR
- PDF_ERROR
- PERMISSION_DENIED
- CANCELLED
- OUTPUT_ERROR
- MODEL_ERROR
