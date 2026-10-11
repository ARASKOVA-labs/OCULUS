# Releasing OCULUS

OCULUS ships as a macOS installer (`.dmg`) for Apple Silicon and Intel. Each installer is
self-contained: the Tesseract and Leptonica libraries and the English, Malayalam and
orientation language data are inside the app, so it runs on a clean Mac.

## Cut a release

1. Set the version in `src-tauri/tauri.conf.json` (and `src-tauri/Cargo.toml`).
2. Commit, then tag and push:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The **Release** workflow builds both architectures, runs the UI and Rust tests, and opens a
   **draft** GitHub release with the `.dmg` files and their `.sha256` checksums.
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
