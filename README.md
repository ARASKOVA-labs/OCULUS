# OCULUS by Araskova

## Purpose
OCULUS is Araskova's universal PDF toolkit for the desktop: every tool you would reach for on iLovePDF, plus OCR with first-class Malayalam support, running entirely on your machine. No uploads, no file-size caps, no daily limits, no paywall on core tools.

## Construction rule
Before writing production code, read DavidStack (`David-Stack/`) if present. It is authoritative for engineering workflow, UI quality, testing discipline and agent behaviour unless it conflicts with explicit project requirements.

## Core architecture
React + TypeScript UI (`araskova-doccraft` submodule)
→ Tauri
→ Rust application layer (`src-tauri/`)
→ C++ native engine (`native/`)
→ Tesseract / Leptonica / lopdf

## Product principles
- Local-first and private: files never leave the machine.
- Universal: one app for organise, convert, edit, secure and OCR.
- Fast on large files and batches.
- Designed in the Araskova house style (see `docs/superpowers/specs/`).

## Running
```bash
cd araskova-doccraft && bun install && bun run dev --port 5173
cd src-tauri && cargo run      # debug builds load the dev server
cd src-tauri && cargo test
```
Production bundle: `bun run build` in the submodule, then `cargo tauri build` and `scripts/bundle_macos.py` (see `PACKAGING.md`).
