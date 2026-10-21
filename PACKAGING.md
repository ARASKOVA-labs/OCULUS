# OCULUS — Packaging & Standalone Distribution Architecture

**Target Platform**: macOS (Apple Silicon arm64 & Intel x86_64)  
**Distribution Model**: Standalone Application Bundle (`Oculus.app` / `.dmg`)  
**Zero-Dependency Guarantee**: 100% self-contained; zero runtime reliance on Homebrew, Xcode command-line tools, MacPorts, or system-level package managers.

---

## 1. Overview & Problem Statement

In standard developer environments, native binaries compiled with Rust and C++ link dynamically against system package manager paths (e.g., `/opt/homebrew/opt/tesseract/lib/libtesseract.5.dylib`). When distributed to an end-user running a clean macOS installation:
```
dyld: Library not loaded: /opt/homebrew/opt/tesseract/lib/libtesseract.5.dylib
  Referenced from: /Applications/Oculus.app/Contents/MacOS/oculus-desktop
  Reason: image not found
```
The application crashes immediately before the entry point or GUI can execute.

To eliminate this barrier, OCULUS employs a deterministic standalone packaging engine (`scripts/bundle_macos.py`) that relocates all transitive dynamic libraries into the application bundle, rewrites load commands to use runtime-relative `@rpath` addressing, embeds language models in `Contents/Resources/tessdata/`, and re-signs all Mach-O binaries.

---

## 2. Dynamic Library Transitive Graph

OCULUS links against Tesseract and Leptonica, which have a deep dependency tree. The complete graph comprises exactly **15 dynamic libraries**:

```
oculus-desktop (Mach-O 64-bit arm64 executable)
 ├── libtesseract.5.dylib
 │    ├── libleptonica.6.dylib
 │    │    ├── libpng16.16.dylib
 │    │    ├── libjpeg.8.dylib
 │    │    ├── libtiff.6.dylib
 │    │    ├── libwebp.7.dylib
 │    │    │    └── libsharpyuv.0.dylib
 │    │    ├── libwebpmux.3.dylib
 │    │    │    └── libsharpyuv.0.dylib
 │    │    ├── libopenjp2.7.dylib
 │    │    ├── libgif.dylib
 │    │    └── libzstd.1.dylib
 │    ├── libarchive.13.dylib
 │    │    ├── libb2.1.dylib
 │    │    ├── liblz4.1.dylib
 │    │    ├── liblzma.5.dylib
 │    │    └── libzstd.1.dylib
 │    └── libcurl.4.dylib (/usr/lib/libcurl.4.dylib — system standard)
 └── libc++.1.dylib (/usr/lib/libc++.1.dylib — system standard)
```

All non-system libraries (15 dylibs) are copied into `Oculus.app/Contents/Frameworks/`.

---

## 3. The Relocation Pipeline (`scripts/bundle_macos.py`)

The automated packaging pipeline executes in 6 deterministic stages:

### Stage 1: Mach-O Layout Creation
Creates the standard macOS Application Bundle directory tree:
```
dist-standalone/Oculus.app/
└── Contents/
    ├── MacOS/
    │   └── oculus-desktop
    ├── Frameworks/
    │   ├── libtesseract.5.dylib
    │   ├── libleptonica.6.dylib
    │   ├── libpng16.16.dylib
    │   ├── libjpeg.8.dylib
    │   ├── libtiff.6.dylib
    │   ├── libwebp.7.dylib
    │   ├── libwebpmux.3.dylib
    │   ├── libsharpyuv.0.dylib
    │   ├── libopenjp2.7.dylib
    │   ├── libgif.dylib
    │   ├── libarchive.13.dylib
    │   ├── libb2.1.dylib
    │   ├── liblz4.1.dylib
    │   ├── liblzma.5.dylib
    │   └── libzstd.1.dylib
    └── Resources/
        └── tessdata/
            ├── eng.traineddata (English LSTM OCR model)
            ├── mal.traineddata (Malayalam LSTM OCR model)
            ├── osd.traineddata (Orientation & Script Detection)
            └── pdf.ttf (Searchable PDF font descriptor)
```

### Stage 2: Transitive Dependency Resolution
Recursively runs `otool -L` across all dependencies starting from `oculus-desktop`.
- Resolves symlinks (e.g. `libtesseract.5.dylib -> libtesseract.5.5.3.dylib`).
- Captures indirect `@rpath` references (such as `libsharpyuv.0.dylib` loaded by `libwebp.7.dylib`).

### Stage 3: Bundle & File Copying
Copies the resolved dynamic libraries into `Contents/Frameworks/` and sets write permissions (`chmod +w`) so Mach-O load commands can be mutated.

### Stage 4: Mach-O Header Rewriting (`install_name_tool`)
1. **Dylib ID (`-id`)**:
   Sets each library's identification name to `@rpath/<basename>`:
   ```bash
   install_name_tool -id @rpath/libtesseract.5.dylib Contents/Frameworks/libtesseract.5.dylib
   ```
2. **Cross-Dylib Load Commands (`-change`)**:
   Rewrites all references pointing to `/opt/homebrew/...` or sibling libraries to `@rpath/<target_basename>`:
   ```bash
   install_name_tool -change /opt/homebrew/opt/leptonica/lib/libleptonica.6.dylib @rpath/libleptonica.6.dylib Contents/Frameworks/libtesseract.5.dylib
   ```
3. **Sibling Lookup RPATH (`-add_rpath`)**:
   Every bundled dylib is injected with:
   ```bash
   install_name_tool -add_rpath @loader_path Contents/Frameworks/<dylib>
   ```
   This ensures that when `libleptonica` requests `@rpath/libpng16.16.dylib`, the dynamic linker searches `@loader_path` (the `Frameworks/` folder itself).

### Stage 5: Executable Configuration
Injects the relative frameworks path into `oculus-desktop`:
```bash
install_name_tool -add_rpath @executable_path/../Frameworks Contents/MacOS/oculus-desktop
install_name_tool -change /opt/homebrew/opt/tesseract/lib/libtesseract.5.dylib @rpath/libtesseract.5.dylib Contents/MacOS/oculus-desktop
install_name_tool -change /opt/homebrew/opt/leptonica/lib/libleptonica.6.dylib @rpath/libleptonica.6.dylib Contents/MacOS/oculus-desktop
install_name_tool -change /opt/homebrew/opt/libarchive/lib/libarchive.13.dylib @rpath/libarchive.13.dylib Contents/MacOS/oculus-desktop
```

### Stage 6: Code Signing & Verification
Because modifying Mach-O headers invalidates existing code signatures:
1. Re-signs all 15 dylibs in `Contents/Frameworks/` with ad-hoc signature:
   ```bash
   codesign -f -s - Contents/Frameworks/*.dylib
   ```
2. Re-signs the main executable:
   ```bash
   codesign -f -s - Contents/MacOS/oculus-desktop
   ```
3. Performs a verification pass ensuring **zero** references to `/opt/homebrew`, `/usr/local`, or non-system paths remain in any binary.

---

## 4. Bundled Offline Assets (`tessdata/`)

To operate without internet access or manual setup:
- `tessdata/` is embedded in `Contents/Resources/tessdata/` containing:
  - `eng.traineddata` (4,113,088 bytes)
  - `mal.traineddata` (5,275,996 bytes)
  - `osd.traineddata` (10,562,727 bytes)
  - `pdf.ttf` (572 bytes)
- In `src-tauri/src/native_engine.rs`, the initialization cascade discovers this directory automatically:
  1. `OCULUS_TESSDATA_PREFIX` / `TESSDATA_PREFIX` environment variables (override for testing).
  2. `<app_dir>/Contents/Resources/tessdata` (bundled macOS app bundle).
  3. `<app_dir>/tessdata` (sibling development directory).
  4. System fallbacks (`/usr/share/tessdata`, `/opt/homebrew/share/tessdata`).

---

## 5. How to Build & Package

Each system has a release script that builds the interface, the app and the installer in one go
(`scripts/release_macos.sh`, `scripts/release_windows.ps1`, `scripts/release_linux.sh`); on macOS it
runs `scripts/bundle_macos.py` on the built app. The installers land in `dist-release/`. See
[RELEASING.md](RELEASING.md) for the prerequisites and for releasing through GitHub Actions.

---

## 6. Linux & Windows Distribution Roadmap

### Linux (AppImage / Debian package) — shipped
`scripts/release_linux.sh` builds both with `tauri build --bundles deb,appimage` (config in
`src-tauri/tauri.linux.conf.json`; see RELEASING.md).
1. **Shared Libraries**: the AppImage bundles `libtesseract.so.4`, `libleptonica` (`liblept.so.5`) and their
   dependencies through `linuxdeploy`. The `.deb` instead depends on the system `libtesseract4` and `liblept5`.
2. **Tessdata**: Tauri installs it to `<prefix>/lib/OCULUS/tessdata`, which the engine's discovery
   cascade (`src-tauri/src/ffi/mod.rs`) checks relative to the executable.
3. **PDF Rasterization**: the native engine has none on Linux (CoreGraphics is macOS-only). The
   interface draws scanned PDF pages with PDF.js and OCRs the images; a native backend (Poppler or
   PDFium) would make long documents faster.

### Windows (MSI / NSIS Installer)
1. **DLL Packaging**: Bundle `tesseract50.dll`, `leptonica-1.82.0.dll`, and MSVC CRT into the application root. Windows searches the executable's directory by default (`SafeDllSearchMode`).
2. **PDF Rasterization**: PDFium DLL linked via CMake.
3. **Tessdata**: Placed in `C:\Program Files\Oculus\tessdata\`.
