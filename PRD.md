# Product Requirements Document — OCULUS

## 1. Product
Name: OCULUS
Organization: Araskova
Category: Local Document Intelligence
Positioning: A privacy-first OCR and document processing workstation.

## 2. Problem
Users need to extract usable text from scans, images and PDFs without uploading confidential documents to a third-party service. Existing tools may be cloud-dependent, difficult to automate, expensive, or poor for batch processing and regional-language workflows.

## 3. Vision
Make high-quality document extraction as simple as dropping files into an application, while keeping the primary processing on the user's device.

## 4. Target users
- Students and researchers
- Developers
- Small businesses
- Operations teams
- Document-heavy professionals
- Dataset builders
- Users processing confidential documents
- Regional-language data teams

## 5. Core user stories
- As a user, I can drag an image into OCULUS and receive editable text.
- As a user, I can OCR a multi-page PDF.
- As a user, I can process hundreds of files in a batch.
- As a user, I can export extracted text as TXT, Markdown, JSON or JSONL.
- As a user, I can create a searchable PDF from a scanned PDF.
- As a user, I can see progress, failures and per-file status.
- As a user, I can preprocess low-quality scans before OCR.
- As a user, I can use Malayalam text without corrupting Unicode.
- As a privacy-conscious user, I can use core OCR without uploading documents.

## 6. Functional requirements

### FR-01 File ingestion
Support PNG, JPG/JPEG, WEBP, TIFF and PDF initially.
Accept drag-and-drop and file picker.
Validate type, size and readability.

### FR-02 Image preprocessing
Provide optional:
- deskew
- grayscale
- denoise
- thresholding
- contrast adjustment
- crop
- rotation
- resolution normalization

### FR-03 OCR
Provide page/image OCR through the native OCR engine.
Architecture must allow replacing or adding OCR backends.
Language packs must be discoverable and configurable.

### FR-04 PDF processing
Detect text-based versus scanned PDFs.
Use direct text extraction when appropriate.
Use OCR for image-only pages.
Preserve page ordering.

### FR-05 Batch jobs
Queue multiple files.
Provide progress.
Allow cancellation.
Continue processing independent files after individual failures.
Provide a final job report.

### FR-06 Export
Support TXT, Markdown, JSON and JSONL.
Preserve page/file metadata in structured formats.
Use UTF-8.

### FR-07 Searchable PDF
Generate a PDF containing the original page image plus an invisible/selectable OCR text layer where technically supported.

### FR-08 Results
Show:
- original file
- page count
- extracted text
- processing duration
- warnings
- OCR confidence when available
- output location

### FR-09 Privacy
Core OCR must work locally.
No document upload by default.
No telemetry by default.
Any future network feature must be explicit and visible.

## 7. Non-functional requirements
- UI remains responsive during processing.
- Large files should be streamed where practical.
- Native work runs outside the UI thread.
- Errors are structured and actionable.
- Architecture supports Windows, macOS and Linux.
- Unicode correctness is mandatory.
- Tests cover critical Rust/C++/IPC boundaries.

## 8. Out of scope for v1
- Mandatory cloud accounts
- Cloud-only OCR
- Online document storage
- Enterprise collaboration
- Automatic legal/financial interpretation
- Fully autonomous document modification
- Training a proprietary OCR model from scratch

## 9. Success criteria
- A user can install/open OCULUS and OCR a document without creating an account.
- Batch OCR works reliably.
- Malayalam text remains valid UTF-8 through the complete pipeline.
- No blocking UI during OCR.
- Native engine failures do not crash the application.
- Core workflow works offline.
