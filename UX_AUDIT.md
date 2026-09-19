# OCULUS — End-to-End User Experience (UX) & Usability Audit

**Audit Date**: September 17, 2026  
**Auditor**: Antigravity Engineering  
**Scope**: Complete end-to-end evaluation of user workflows across all 5 desktop application routes.

---

## 1. Single Document Workflow (Drop $\rightarrow$ OCR $\rightarrow$ Review $\rightarrow$ Edit $\rightarrow$ Export)

### Step 1: Ingestion & Drop Zone
- **Observed Behavior**: Dragging and dropping a document (PNG, JPEG, PDF) into the workspace drop zone triggers `inspect_document`.
- **Validation**: File size and magic bytes are verified. Files exceeding 250 MB display an immediate warning badge: *"File exceeds safety limit of 250MB"*.
- **Empty State**: A prominent, accessible dropzone with clear SVG iconography and format hints (`PNG, JPG, PDF up to 250MB`) prevents user confusion.

### Step 2: Preprocessing Controls & OCR Trigger
- **Observed Behavior**: All 6 preprocessing toggles (Orientation, Deskew, Grayscale, Denoise, Contrast Normalization, Otsu Threshold) update reactive state in `OculusContext`.
- **Feedback**: Clicking *"Run OCR"* transitions the UI state to `PROCESSING`, displaying a real-time progress bar with active stage messages (*"Preprocessing image"*, *"Rasterizing page 1/3"*, *"Recognizing text"*).

### Step 3: Result Review & Bounding Box Inspection
- **Observed Behavior**: Extracted text renders in an accessible typography pane with full Malayalam Unicode shaping (no tofu glyphs, conjuncts properly rendered).
- **Bounding Boxes**: Toggling *"Show Bounding Boxes"* overlays colored geometric highlights on recognized text blocks with confidence percentages.
- **Confidence Badging**: Pages display green ($> 80\%$), yellow ($50 - 80\%$), or red ($< 50\%$) confidence badges.

### Step 4: Export Actions
- **Observed Behavior**: Dedicated export actions for Searchable PDF (`.pdf`), Plain Text (`.txt`), Markdown (`.md`), JSON (`.json`), and JSON Lines (`.jsonl`).
- **Feedback**: Export triggers native file-save feedback with byte counts written and directory destination.

---

## 2. Batch Processing Workflow (Drop 100 Files $\rightarrow$ Batch $\rightarrow$ Monitor $\rightarrow$ Cancel $\rightarrow$ Review Failures $\rightarrow$ Export)

### Step 1: Queue Population
- **Observed Behavior**: Dropping multiple files populates the document queue with individual status badges (`IDLE`).
- **Duplicate Handling**: Duplicate files are permitted in the queue with distinct UUIDs to allow reprocessing under different preprocessing profiles.

### Step 2: Live Monitoring
- **Observed Behavior**: Clicking *"Process All"* launches `start_batch_ocr`.
- **Banner Feedback**: A persistent banner displays:
  - Total files completed vs pending (`X / Y`).
  - Active filename currently being processed.
  - Overall completion percentage bar.

### Step 3: Cancellation Responsiveness
- **Observed Behavior**: Clicking *"Cancel Batch"* sets the atomic cancel token.
- **Latency**: Interrupted in $< 1\text{ ms}$. In-flight workers finish their current single-page slice and abort subsequent tasks.
- **State Transition**: State updates to `CANCELLED` with a grey warning badge.

### Step 4: Failure Review & Isolation
- **Observed Behavior**: Failed files display an `ERROR` badge with hoverable tooltips displaying the exact failure cause (e.g. *"File not found"*, *"Corrupted PDF structure"*). Succeeded files remain accessible and exportable.

---

## 3. Usability Gaps Discovered & Resolved

| Component | Issue Identified | UX Impact | Resolution Applied |
| :--- | :--- | :--- | :--- |
| **Export Dialog** | Searchable PDF failed on image inputs | User confusion when clicking "Export PDF" on a PNG | Fixed `export_result` to pass raster image bytes directly to `create_searchable_pdf`. |
| **Path Resolution** | Basename stripping broke exports | "File not found" error during export if launched from non-root directory | Preserved absolute canonical paths in domain model `OcrResult.filename`. |
| **Key-Value Extraction** | Missed full-width colons (`：`) and equals (`=`) | Scanned receipts failed to populate the Extract table | Expanded delimiter parser in `DocumentIntelligence::extract_fields` and added adjacent-block pairing. |
| **Error Feedback** | Corrupted multi-page PDFs aborted the entire document | One bad page lost 49 good pages | Implemented page-level fault isolation: bad pages insert warning placeholders without killing the document. |

---

## 4. Accessibility (a11y) Evaluation

- **Color Contrast**: All text elements meet WCAG AA standards (4.5:1 ratio against dark slate background).
- **Semantic HTML**: Buttons, inputs, and headings use proper HTML5 semantic elements (`<button>`, `<input type="checkbox">`, `<h1>`-`<h3>`).
- **Screen Reader Support**: Bounding box counts and progress states include descriptive `aria-label` attributes.
- **Keyboard Navigation**: Tab key navigates cleanly across sidebar links, file drop triggers, and preprocessing checkboxes.
