# Releasing OCULUS

OCULUS ships as a macOS installer (`.dmg`) for Apple Silicon and Intel, plus a Windows installer
and Linux packages (see the Windows and Linux sections below). Each macOS installer is
self-contained: the Tesseract and Leptonica libraries and the English, Malayalam and
orientation language data are inside the app, so it runs on a clean Mac.

## Cut a release

1. Set the version in `src-tauri/tauri.conf.json` (and `src-tauri/Cargo.toml`).
2. Commit, then tag and push:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The **Release** workflow builds both macOS architectures, Windows and Linux, runs the UI and Rust
   tests, and opens a **draft** GitHub release with the `.dmg` / `-setup.exe` / `.AppImage` / `.deb`
   files and their `.sha256` checksums.
4. Open the draft, check the installers, then publish.

The workflow refuses to run if the tag does not match the version in `tauri.conf.json`.
To rehearse without publishing, use **Actions → Release → Run workflow**: it builds and attaches
the installers as workflow artifacts only.

## Repository secrets

| Secret | Needed | What it is |
|---|---|---|
| `SUBMODULE_TOKEN` | While the repos are private | Fine-grained token with read access to `ARASKOVA-labs/araskova-doccraft` (a private UI submodule is not readable with the default workflow token). Once both repositories are public the workflows fall back to the default token and the secret can be deleted. |
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

**Scanned PDFs:** the native engine turns PDF pages into images with macOS CoreGraphics, which
Windows does not have. When it reports that, the interface draws each page with PDF.js and sends
the page images to the native OCR instead (`ocrPdfAsImages` in `araskova-doccraft/src/lib/tauri-bridge.ts`).
It is slower than macOS on long documents.

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

## Linux (x64)

The same tag also builds Linux packages on an Ubuntu 22.04 runner: `OCULUS_<version>_amd64.deb` and
`OCULUS_<version>_amd64.AppImage`, each with a `.sha256`. The language data is installed next to the
app (`/usr/lib/OCULUS/tessdata` for the `.deb`, inside the image for the AppImage); the app finds it
without any setup.

- **AppImage** carries its own copies of Tesseract, Leptonica and the GTK/WebKit libraries, so it runs
  on x86-64 distributions from 2022 or newer (Ubuntu 22.04, Debian 12, Fedora 36 and later). Make it
  executable (`chmod +x OCULUS_*.AppImage`) and run it.
- **.deb** is for Ubuntu 22.04 and derivatives. It depends on the system `libtesseract4` and
  `liblept5` (Ubuntu 22.04 ships Tesseract 4.1), plus WebKitGTK 4.1 and GTK 3, and `apt` pulls them in:
  `sudo apt install ./OCULUS_*.deb`. Distributions that ship Tesseract 5 need the AppImage instead.

**The Linux build is best-effort**, like Windows: it runs in its own job that is allowed to fail, and
the draft notes say so if the Linux packages are missing.

**What works on Linux:** image OCR (PNG, JPG, TIFF) in English and Malayalam, and all the
browser-side tools. Scanned PDFs are read the same way as on Windows: the interface draws the pages
with PDF.js and the native OCR reads the images.

Linux packages are not signed.

### Build locally on Linux

On Ubuntu 22.04 (a container works too):

```bash
sudo apt install build-essential pkg-config patchelf libssl-dev libxdo-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  libtesseract-dev libleptonica-dev
(cd araskova-doccraft && bun install --frozen-lockfile)
scripts/release_linux.sh        # writes dist-release/OCULUS_<version>_<arch>.deb and .AppImage
```

The Linux configuration is in `src-tauri/tauri.linux.conf.json`. On a machine without a display or
FUSE (a container, CI) the script already sets `APPIMAGE_EXTRACT_AND_RUN=1`. On Apple Silicon the same
steps in an `arm64` Ubuntu 22.04 container produce `arm64` / `aarch64` packages, which is a quick way
to check the Linux build; only the x64 packages are released.

## Continuous integration

The **CI** workflow (`.github/workflows/ci.yml`) runs on every push to `main` and on pull requests;
a newer push cancels the older run. It runs:

- **UI** (Linux): `bun install --frozen-lockfile`, lint, the UI tests, and the static desktop build.
- **Native** (macOS, Ubuntu 22.04 and Windows): the Rust and C++ build, then `cargo test`, with
  Tesseract and Leptonica installed the same way as in the release workflow (Homebrew, `apt`, vcpkg).
  The Windows tests are informational, as in the release; its build is not.

CI needs no secrets once the repositories are public. While they are private it uses
`SUBMODULE_TOKEN` to read the UI submodule. Cargo builds, the vcpkg packages and bun downloads are
cached, so a run after the first is much faster.

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
