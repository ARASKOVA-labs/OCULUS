# OCULUS Test Plan

## Unit tests
Rust:
- validation
- job state
- export
- path handling
- configuration
- error mapping

C++:
- preprocessing
- OCR wrapper
- image transformations
- memory/resource lifecycle

## Integration tests
- React → Tauri → Rust
- Rust → C++ FFI
- OCR → export
- PDF → OCR → searchable PDF

## Test corpus
Maintain samples for:
- clean scans
- skewed scans
- low contrast
- noisy scans
- multi-page PDFs
- large PDFs
- Malayalam documents
- mixed-language documents
- empty pages
- corrupted files

## Performance tests
Measure:
- startup time
- OCR throughput
- memory usage
- batch throughput
- large PDF handling
- cancellation latency

## Reliability requirements
- A single bad file must not terminate the batch.
- Cancellation must leave the workspace clean.
- Native engine crashes/errors must be isolated as far as practical.
- Output files must never be partially reported as successful.
