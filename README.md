# OCULUS

**Every PDF tool. None of the uploads.**

OCULUS is a desktop PDF toolkit by [Araskova](https://araskova.com). Merge, split, compress,
convert, protect, repair and read PDFs, including Malayalam OCR, entirely on your own computer.
Files are never uploaded. The app makes **no network requests**.

## What it does

| | |
|---|---|
| **Organize** | Merge, split, remove pages, extract pages, reorder |
| **Optimize** | Compress, repair |
| **Edit** | Rotate, page numbers, watermark |
| **Security** | Protect with a password, unlock |
| **Convert** | PDF to Word, PowerPoint, Excel, JPG, PNG; Word, PowerPoint, Excel, JPG, PNG to PDF; image and e-book formats |
| **OCR & Extract** | Read text from scans and images (English, Malayalam and more), extract tables and form fields |

Press <kbd>⌘K</kbd> (or <kbd>Ctrl</kbd>+<kbd>K</kbd>) to search every tool, or drop a file on the
home screen and OCULUS suggests what to do with it.

## Install

Download the installer for your system from the
[Releases](https://github.com/ARASKOVA-labs/OCULUS/releases) page:

- **macOS** (Apple Silicon and Intel): open the `.dmg` and drag OCULUS to Applications. Until the
  app is notarised, macOS warns on first launch: right-click the app and choose **Open**.
- **Windows** (x64, preview): run the `-setup.exe`. PDF-page OCR needs the macOS build for now; all
  other tools work. Until the installer is signed, SmartScreen may warn: **More info, Run anyway**.

Each release lists SHA-256 checksums so you can verify the download.

## Build from source

You need Rust, [Bun](https://bun.sh), and Tesseract and Leptonica (`brew install tesseract leptonica
pkg-config` on macOS).

```bash
git clone --recurse-submodules https://github.com/ARASKOVA-labs/OCULUS.git
cd OCULUS
(cd araskova-doccraft && bun install && bun run dev --port 5173) &   # the UI
(cd src-tauri && cargo run)                                          # the desktop shell
```

Tests: `cd araskova-doccraft && bun run test` for the UI, `cd src-tauri && cargo test` for the
native side. To build an installer, see [RELEASING.md](RELEASING.md).

## How it is built

```
React + TypeScript UI  (araskova-doccraft/)
   -> Tauri             (src-tauri/)
   -> Rust app layer    (jobs, files, IPC)
   -> C++ engine        (native/: Tesseract + Leptonica)
```

The interface follows a strict visual system (ink and bone, one orange accent, hairlines, square
corners). [CONTRIBUTING.md](CONTRIBUTING.md) explains the rules.

## Contributing

Contributions are welcome: bug reports, fixes, translations, language data, and tools. Start with
[CONTRIBUTING.md](CONTRIBUTING.md). Please report security problems privately; see
[SECURITY.md](SECURITY.md).

## License

OCULUS is licensed under the **Functional Source License, Version 1.1, with an Apache 2.0 future
license** ([FSL-1.1-ALv2](LICENSE)). In plain words:

- You can use it, read it, modify it and share it, including for your own work and your company's
  internal use.
- You **cannot** use it to build and sell a competing product or service.
- Two years after each version is released, that version becomes available under the Apache License
  2.0 with no restrictions.

This is a source-available licence, not an OSI-approved open-source licence. The names **Araskova**
and **OCULUS** and the Rephen O logo are trademarks of Araskova; see [TRADEMARKS.md](TRADEMARKS.md).
Third-party components keep their own licences; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
