# OCULUS — Local Document Intelligence

## Purpose
OCULUS is an Araskova local-first document intelligence product for OCR, document preprocessing, structured extraction, searchable PDFs, dataset generation, and optional local AI features.

## Construction rule
Before writing production code, the implementation agent MUST inspect the project folder for `DavidStack/`, `davidstack/`, or similarly named DavidStack files. DavidStack is a project-local engineering rule stack that will be added later. When present, it is authoritative for product construction quality, engineering workflow, UI quality, testing discipline, and agent behavior unless it conflicts with explicit project requirements or higher-priority system constraints.

Do not invent DavidStack rules if the files are not yet present. If absent, follow this repository's documents and record that DavidStack is pending.

## Core architecture
React + TypeScript UI
→ Tauri
→ Rust application layer
→ C++ native engine
→ OCR / OpenCV / optional ONNX Runtime

## Product principle
Local-first, privacy-first, fast, accessible, free for core functionality, and capable of running without a cloud backend.

## Initial scope
1. Image OCR
2. PDF OCR
3. Image preprocessing
4. Text preview and correction
5. TXT/Markdown/JSON/JSONL export
6. Searchable PDF generation
7. Batch processing
8. Progress and job management
9. OCR confidence information where available
10. English + Malayalam-ready text pipeline

Cloud AI and advanced document intelligence are optional later layers, not dependencies for core OCR.
