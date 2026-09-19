# Architecture Decision Record

## ADR-001: Rust is the application layer
Decision: Rust owns orchestration, filesystem, networking, jobs, configuration, IPC and security-sensitive application logic.

Reason: memory safety, concurrency, strong type system, Tauri integration and cross-platform application development.

## ADR-002: C++ is the native engine
Decision: C++ is used for DSP, OpenCV, FFmpeg, OCR and other areas where native ecosystem compatibility or performance matters.

Reason: mature computer vision/media/native ML ecosystem.

## ADR-003: Local-first
Decision: Core OCR does not require a cloud backend.

Reason: privacy, cost, offline capability and predictable operation.

## ADR-004: Web edition is secondary
Decision: A browser/WASM deployment may be produced after the native pipeline is stable.

Reason: native desktop access is the most reliable path for large files and native libraries; WASM compatibility can be evaluated independently.
