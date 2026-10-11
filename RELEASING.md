# Releasing OCULUS

OCULUS ships as a macOS installer (`.dmg`) for Apple Silicon and Intel, plus a Windows installer
(see the Windows section below). Each macOS installer is
self-contained: the Tesseract and Leptonica libraries and the English, Malayalam and
orientation language data are inside the app, so it runs on a clean Mac.

## Cut a release

1. Set the version in `src-tauri/tauri.conf.json` (and `src-tauri/Cargo.toml`).
2. Commit, then tag and push:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The **Release** workflow builds both macOS architectures and Windows, runs the UI and Rust
   tests, and opens a **draft** GitHub release with the `.dmg` / `-setup.exe` files and their
   `.sha256` checksums.
4. Open the draft, check the installers, then publish.

The workflow refuses to run if the tag does not match the version in `tauri.conf.json`.
To rehearse without publishing, use **Actions → Release → Run workflow**: it builds and attaches
the installers as workflow artifacts only.

## Repository secrets

| Secret | Needed | What it is |
|---|---|---|
| `SUBMODULE_TOKEN` | **Yes** | Fine-grained token with read access to `ARASKOVA-labs/araskova-doccraft` (the UI submodule is private; the default workflow token cannot read it). |
| `APPLE_CERTIFICATE` | No | Base64 of a *Developer ID Application* `.p12` certificate. |
| `APPLE_CERTIFICATE_PASSWORD` | With the certificate | Password of that `.p12`. |
| `APPLE_SIGNING_IDENTITY` | With the certificate | e.g. `Developer ID Application: Araskova (TEAMID)`. |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | No | Apple ID, an app-specific password, and the team id, for notarisation. |
| `WINDOWS_CERTIFICATE` | No | Base64 of a code-signing `.pfx` certificate (Authenticode). |
| `WINDOWS_CERTIFICATE_PASSWORD` | With the certificate | Password of that `.pfx`. |

Without the Apple secrets the app is **ad-hoc signed**. It runs, but macOS Gatekeeper warns on
first launch: right-click the app and choose **Open** (or run
`xattr -dr com.apple.quarantine /Applications/OCULUS.app`). With the certificate and notarisation
secrets the installer opens without a warning.

## Build locally

```bash
brew install tesseract leptonica pkg-config
(cd araskova-doccraft && bun install --frozen-lockfile)
scripts/release_macos.sh        # writes dist-release/OCULUS_<version>_<arch>.dmg
```

## Windows (x64)

The same tag also builds a Windows installer: `OCULUS_<version>_x64-setup.exe` (NSIS, per-user
install, no administrator rights needed) with its `.sha256`. Tesseract and Leptonica are linked
statically, the language data is installed next to the app (`<install dir>\tessdata`), and the
WebView2 runtime is bootstrapped by the installer if the machine lacks it.

**The Windows build is best-effort.** It runs in its own job that is allowed to fail: if it fails,
the macOS release is still drafted, and the draft notes say the Windows build is missing. The
failed run uploads `windows-build-logs` (vcpkg and build-script logs) as an artifact.

**What works on Windows:** image OCR (PNG, JPG, TIFF) in English and Malayalam, and all the
browser-side tools (merge, split, compress, convert and the rest). Text-based PDFs are read through
their text layer.

**What does not work yet:** OCR of scanned PDF pages. Turning a PDF page into an image uses macOS
CoreGraphics, and there is no replacement on Windows in this release. OCULUS shows a clear message
instead (convert the PDF to JPG with the PDF to JPG tool and run OCR on the images, or use macOS).
The draft release notes the workflow writes say the same.

**SmartScreen:** without a certificate the installer is unsigned, so Windows shows "Windows
protected your PC". Choose **More info**, then **Run anyway**. Add `WINDOWS_CERTIFICATE` and
`WINDOWS_CERTIFICATE_PASSWORD` to sign the installer with `signtool`; signing happens after the
build, so the installer is signed but the app inside it is not. SmartScreen reputation for a new
certificate still takes time to build.

### Build locally on Windows

Needs Visual Studio Build Tools (C++ workload), Rust (MSVC toolchain), bun and vcpkg.

```powershell
vcpkg install "tesseract[core]:x64-windows-static-md"
$env:VCPKG_ROOT = "C:\path\to\vcpkg"
cd araskova-doccraft; bun install --frozen-lockfile; cd ..
scripts\release_windows.ps1      # writes dist-release\OCULUS_<version>_x64-setup.exe
```

The Windows configuration is in `src-tauri/tauri.windows.conf.json`. The script logs to
`build-logs\release_windows.log`.

## What the release build does for security

- Rust is built with `lto = "fat"`, `strip = "symbols"` and `panic = "abort"`; the build machine's
  folder names are remapped out of the binary.
- The UI is minified and ships **no source maps**; devtools are off in release builds.
- The window enforces a strict Content-Security-Policy (`src-tauri/tauri.conf.json`): same-origin
  scripts only, no remote content, no framing.
- The app makes no network requests. The only native capabilities are the file dialog and the
  commands the UI calls (`src-tauri/capabilities/default.json`).
- Developer ID builds use the hardened runtime and a secure timestamp.

None of this makes a desktop app impossible to inspect. A determined person can always read what
runs on their own machine. It raises the cost of casual copying; it is not a secret store.
Keep keys and licensing secrets out of the app.
