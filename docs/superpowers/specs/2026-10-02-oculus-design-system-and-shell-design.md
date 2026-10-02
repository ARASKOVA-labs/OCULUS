# OCULUS design system and app shell: design

Sub-project 2 of 4 in the OCULUS pivot (1 cleanup ✓, 2 design system + shell, 3 tool catalogue, 4 differentiators).

## Goal

Redesign OCULUS from the current stark-white "iLovePDF clone" into an Araskova product in Anvin's aesthetic, so it no longer looks like a clone. It stays a Tauri desktop app and keeps all 33 working tools and their engines. This sub-project changes how the app looks, how you move through it and its branding. It adds no tools.

## What the user asked for

- Keep Tauri. Drop the cloud/PWA idea.
- Redesign the UI completely in "Anvin's aesthetics" (`~/Documents/Projects-Araskova/anvins-aesthetics`, formerly pdoom) with Araskova branding.
- Become a universal PDF tool that beats iLovePDF in every way. That is sub-projects 3 and 4. This one lays the foundation they build on.

## Assumptions (made by Claude, user said "do what is best")

- Selling point over iLovePDF: local, private, unlimited and free. No uploads, no caps, no paywall. The `pricing` route goes.
- Dark-first. The aesthetic is ink-field. A bone "paper" theme exists for people who want light mode, and documents are always previewed on bone paper.
- The worked example is the Araskova Extractor desktop app, which uses the same tokens and fonts.

## Sources

- `anvins-aesthetics/docs/TREATMENT.md`: tone, palette, typography, motifs (the spark and its line, crop marks, deadpan mono annotations, stamps).
- Araskova tokens (araskova.com): `#0A0A0A`, `#F3F3F2`, `#81868B`, `#E73F07`. Space Mono and Roboto Flex.
- Araskova mark: rounded-apex chevron, `Araskova-final/public/images/araskova-logo.png`, redrawn here as SVG.

## 1. Tokens

| token | dark (default) | paper (light) | use |
|---|---|---|---|
| `--field` | `#0A0A0A` | `#F3F3F2` | app background |
| `--panel` | `#151517` | `#EAE9E6` | raised surfaces |
| `--rule` | `#2A2A2A` | `#D4D3CF` | hairlines (1px) |
| `--rule-strong` | `#3A3A3A` | `#B9B8B3` | focused or hovered hairlines |
| `--ash` | `#81868B` (5.39:1) | `#5E6266` (5.54:1) | secondary text, mono annotations |
| `--type` | `#F3F3F2` | `#0A0A0A` | primary text |
| `--signal` | `#E73F07` (4.82:1 on field) | `#C2340A` (4.99:1) | the only hue: accents, primary action, progress, errors |
| `--signal-ink` | `#0A0A0A` | `#F3F3F2` | text on a signal fill (dark: 4.82:1) |
| `--paper` | `#F3F3F2` | `#FFFFFF` | page previews (documents always on paper) |

Hard rules:
- **One hue.** There is no green, blue, red or yellow anywhere, including success, warning and error states. Status is carried by mono labels and glyphs:
  - `DONE ✓` in `--type`
  - `ERR` in `--signal` with a hairline signal border
  - `WARN` in `--ash` with a dashed border
- Only `--signal` may glow (a soft box-shadow bloom on the spark and the active progress head). Type never glows.
- Radius is 0 for frames and panels and 2px for inputs and buttons. No pills, no drop shadows, no gradients, no glassmorphism.
- Theme: `data-theme="dark|paper"` on `<html>`, default dark, persisted in settings, with no system auto-switch (the aesthetic is dark-first).

## 2. Typography

All fonts are bundled locally through `@fontsource`. The current Google Fonts `<link>` is removed: an offline, private app must not call out to Google.

- **Roboto Flex** (variable, width axis) is the display and UI voice. Display headings are big and tight (`wdth` 25–151, weight 300–900, tracking −0.02em). UI text is 14px/1.5.
- **Space Mono** is the machine voice: the wordmark, labels, tool indices (`T-07`), format tags (`PDF → DOCX`), counters, statuses and the command bar. Uppercase, tracking 0.12–0.22em.
- **Noto Sans Malayalam** (variable) for any Malayalam text in previews and OCR output.
- Tabular figures for every number. Typographic punctuation in prose. Typewriter quotes are allowed only in mono input echoes.

## 3. Brand

- **Mark:** the Araskova chevron as an inline SVG component `<AraskovaMark>` (stroke-based, `currentColor`) in `--type`.
- **Wordmark:** `OCULUS` in Space Mono 500, tracking 0.22em, followed by a `--signal` full stop (`OCULUS.`), with `by Araskova` in `--ash`.
- **App icon:** the chevron in `--signal` on an ink square. Regenerate `src-tauri/icons/*` (png, icns, ico) from one 1024px SVG source committed at `src-tauri/icons/source.svg`.
- **Window title:** `OCULUS — Araskova`.
- **Copy voice:** precise and deadpan, with small footnotes. Banned words: "neural", "magic", "AI-powered", "seamless", "unleash". There are no emoji. Example hero: "Every PDF tool. / None of the uploads." with a mono footnote: "¹ files processed on this machine. Network calls: 0."

## 4. Motion

- Strong eases: `cubic-bezier(.16,1,.3,1)` (outExpo) for entrances and `cubic-bezier(.65,0,.35,1)` for travel. Holds, then snaps. No floaty loops.
- **The spark:** an orange point with a bloom, dragging a hairline. It is the single progress idiom: indeterminate, determinate per file, and batch (one tick per file on the line). It is reused from the Extractor's loading screen.
- Crop marks frame the drop zone and close in on a file drop (a 300ms snap).
- `prefers-reduced-motion` turns off every animation. Progress becomes a static filled line.

## 5. Shell and screens

### Top bar (48px, hairline bottom)
- Left: mark and wordmark (links home).
- Centre: the **command bar** `> find a tool, or drop a file` (⌘K). It opens a command palette over the tool registry with fuzzy search on names, aliases and formats ("docx", "shrink", "combine").
- Right: the status `● LOCAL · 0 UPLOADS` in mono (the dot is signal), the theme toggle and a settings link.

### Home (`/`)
1. Hero: Roboto Flex display line, a mono footnote and a hairline rule.
2. **Drop zone:** full-width hairline frame with crop marks.
   - **Smart routing:** when files are dropped, the zone shows only the tools that accept those file types, ranked by usefulness. For example, dropping PDFs offers Merge, Compress, OCR, then the rest. A file you drop stays loaded when you pick a tool.
3. **Tool index:** numbered sections (`01 ORGANIZE`, `02 OPTIMIZE`, `03 EDIT`, `04 SECURITY`, `05 FROM PDF`, `06 TO PDF`, `07 IMAGE`, `08 OTHER`, `09 OCR & DATA`).
   - Each tool is a hairline cell holding its index (`T-01`), name, one line of description and format tag (`PDF → DOCX`).
   - Hover lights the cell's top rule in signal and nudges the index.
   - The grid is 4 columns at desktop width, 2 at 800px (the window's minimum).

### Tool workspace (`/tool/$toolId`, replaces `/convert/$toolId`)
Three regions with a sticky action bar:
- **Left, files:** the input stack with thumbnails. Reorder by drag, remove, add more. Each file shows its page count and size in mono.
- **Centre, preview:** pages rendered on `--paper` (pdf.js), hairline page frames and mono page numbers. Tools that act on pages (organize, remove, extract, rotate) select and act directly on these thumbnails.
- **Right, options:** the tool's options as labelled controls. The tool's format tag and a short "what happens" note sit at the top.
- **Action bar:** a primary `RUN ⏎` button (signal fill, ink text). It turns into the spark progress line while running. When finished there is a result row: file name, size delta in mono (`−62%` for compress), `Show in folder`, `Open`, `Run again`.
- **Errors:** inline `ERR` block with the engine's message and a "copy details" control. No modal dialogs for errors.

### Other routes
- Today `ocr`, `extract`, `datasets` and `settings` are only redirects to sections of the 845-line home page (`index.tsx`), which holds those flows inline. They move out of the home:
  - OCR, Extract and Dataset export become tools in a new index section `09 OCR & DATA`. They open in the tool workspace and keep their current Tauri commands (`run_ocr`, `analyze_document`, `export_result`, `start_batch_ocr`).
  - `/settings` becomes a real page (theme, OCR languages, default output folder) instead of a home anchor.
- `editor` and `forms` are restyled onto the new shell and components with no behaviour change.
- `pricing` is deleted (the app is free and local).
- `/convert` and `/convert/$toolId` redirect to `/` and `/tool/$toolId`.

## 6. Component layer

- Replace the stark-white token block in `src/styles.css` (2,869 lines) with the new tokens mapped into Tailwind v4 `@theme`. Delete the page-specific CSS that the new components replace, with a target under 600 lines.
- Restyle the shadcn primitives in `src/components/ui/` through tokens only (button, input, select, switch, tabs, dialog, tooltip, dropdown, scroll-area). Delete the unused primitives `carousel`, `calendar`, `chart`, `menubar`, `navigation-menu`, `pagination`, `breadcrumb` and `context-menu` (checked 2026-10-02: no imports outside `components/ui`).
- New components: `AraskovaMark`, `Wordmark`, `CommandPalette`, `DropZone` (crop marks and smart routing), `ToolCell`, `SparkProgress`, `StatusTag` (`DONE`/`ERR`/`WARN`/`RUN`), `FileStack`, `PagePreview`, `ResultRow`.
- `oculus-shell.tsx` (770 lines) is split into `TopBar`, `CommandPalette` and the route layout, with no file over about 300 lines.
- The tool registry (`converter-registry.ts`) gains `index` (T-number), `aliases`, `accepts` (MIME and extensions) and `rank`, which the palette and smart routing use. Engines (`converter-engine.ts`, `pdf-tools-engine.ts`, `tauri-bridge.ts`) are untouched.

## 7. Where the work happens

- The UI lives in the `araskova-doccraft` submodule (github.com/ARASKOVA-labs/araskova-doccraft). Initialise it in this worktree and work on a branch `feat/araskova-redesign` there. The parent repo bumps the submodule pointer and carries the icon and window-title changes.
- No Rust or C++ changes, apart from `tauri.conf.json` (title) and icons.

## 8. Testing and verification

- `bun run build`, `bun run lint` and `tsc --noEmit` pass. The DavidStack banned patterns (no `any`, no direct DOM manipulation) hold in new code.
- In the browser preview, screenshot and check:
  - home in both themes, at 1280px and 800px
  - the command palette finding "shrink" → Compress PDF
  - dropping `test_samples/sample_digital.pdf` routes to PDF tools
  - Merge PDF and Compress PDF run end to end and produce output
  - an error state (encrypted PDF to a tool that cannot open it)
- In the desktop app, `cargo run` with the dev server: the window shows the new home, the icon and title are updated, and a Tauri-backed tool (Repair PDF) runs.
- Check contrast for every text and background pair in the token table (values above are measured, WCAG AA ≥ 4.5:1). Keyboard: Tab order through the tool workspace, ⌘K, Esc and ⏎ to run.

## Out of scope

- New tools and the "better than iLovePDF" catalogue (sub-project 3).
- Tool chaining or pipelines, batch recipes and Malayalam OCR improvements (sub-project 4).
- Backend or engine changes, Windows and Linux builds, auto-update and localisation of the UI.
