# Contributing to OCULUS

Thank you for helping. OCULUS is a local-first PDF toolkit: files stay on the user's machine, and
the interface follows a strict visual system. This guide covers how to contribute and what makes a
change easy to accept.

## Ground rules

1. **Privacy first.** The app makes no network requests and never uploads files. A change that
   adds network access, telemetry, analytics or remote code is not accepted.
2. **PDF tools only.** OCULUS is a PDF and document toolkit. Features that build datasets or
   training data from documents are out of scope.
3. **Be kind.** See the [Code of Conduct](CODE_OF_CONDUCT.md).
4. **Security issues go private.** See [SECURITY.md](SECURITY.md). Do not open a public issue.

## Before you start

- Look for an existing issue. For anything larger than a fix, open an issue first so we can agree on
  the approach before you spend time on it.
- Small fixes (typos, bugs, tests, docs) can go straight to a pull request.

## Set up

You need Rust, [Bun](https://bun.sh), and Tesseract and Leptonica (`brew install tesseract leptonica
pkg-config` on macOS).

```bash
git clone --recurse-submodules https://github.com/YOUR-NAME/OCULUS.git
cd OCULUS
cd araskova-doccraft && bun install
bun run dev --port 5173            # the UI, in one terminal
cd ../src-tauri && cargo run       # the desktop shell, in another
```

Run the checks before you open a pull request:

```bash
cd araskova-doccraft && bun run test && bun run lint && bun run build
cd ../src-tauri && cargo test
```

## Making a change

1. Fork, then create a branch: `git switch -c fix/short-description`.
2. Make the change with tests. New behaviour needs a test; a bug fix needs a test that fails
   without the fix.
3. Keep the pull request focused: one change, a clear description, screenshots for any visual change
   (light and dark theme).
4. **Sign off every commit** (`git commit -s`). This adds a `Signed-off-by` line certifying the
   [Developer Certificate of Origin](https://developercertificate.org): you wrote the change or
   have the right to submit it.
5. Open the pull request against `main`.

## Code rules

- TypeScript: no `any`; use specific types or `unknown` with a guard. No direct DOM queries
  (`getElementById`, `querySelector`, `createElement`) in React code; use refs and state.
- No `TODO` markers left in merged code.
- Files stay under about 300 lines; split by responsibility.
- Rust: `cargo fmt`, no new Clippy warnings, no `unsafe` outside the C++ boundary
  (`src-tauri/src/ffi`), and validate every path that comes from the UI.
- Never log document contents.
- Do not edit the engine files in `araskova-doccraft/src/lib/` (`converter-engine.ts`,
  `pdf-tools-engine.ts`, `tauri-bridge.ts`) without discussing it first; many tools depend on them.

## Design rules

The look is deliberate. A change that breaks these is sent back, however good the code is.

- **Palette:** ink `#0A0A0B`, bone `#EEE9DF`, graphite, ash, and **one** accent, signal orange
  (`#FF4D12`; `#B83204` in the light theme). No other colours, including for success and error.
  Status is a word or a stamp, not a colour.
- **Shape:** square corners, 1 px hairlines, no shadows or gradients (only the spark's glow).
- **Type:** Archivo for display, IBM Plex Mono for labels, Noto Sans Malayalam for Malayalam text.
  All fonts are bundled; never load fonts from the network.
- **Themes:** every surface follows the light and dark theme. Use the theme tokens; do not hard-code
  colours or add always-dark panels.
- **Motion:** purposeful and short. No idle looping animation, no flashes. Respect
  `prefers-reduced-motion`. The boot screen is the only place for a glitch effect.
- **Copy:** plain, short, and calm. No emoji. Avoid hype words (neural, magic, seamless).
- **Icons:** Lucide icons, drawn at the size of the text beside them.

## Adding a tool

1. Add it to `araskova-doccraft/src/lib/converter-registry.ts` (name, formats, icon, description) and
   `tool-meta.ts` (family, aliases, rank).
2. Implement it behind the shared tool screen; do not build a one-off page.
3. Make the receipt honest: the stamp must describe what really happened.
4. Add tests for the options mapping and the result, and check it in both themes.

## Languages and OCR data

Better Malayalam (or other language) OCR is especially welcome: test documents, regression cases
and traineddata improvements. Add test images under `test_samples/` only if you have the right to
share them.

## License of contributions

By contributing you agree that your contribution is licensed under the same terms as the project
([FSL-1.1-ALv2](LICENSE)). You keep your copyright.
