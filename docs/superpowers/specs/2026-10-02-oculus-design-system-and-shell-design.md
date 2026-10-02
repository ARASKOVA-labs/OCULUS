# OCULUS design system and app shell: design (v2, bible-driven)

Sub-project 2 of 4 in the OCULUS pivot (1 cleanup ✓, 2 design system + shell, 3 tool catalogue, 4 differentiators).

v2 (2026-10-02): the user asked for the UI to come from **Anvin's aesthetic bible**, not from the OCULUS docs. v1 was a generic three-pane tool app (shadcn, iLovePDF grid, "calm workstation" from the old UI_SPEC) with only the bible's colours. v2 replaces it.

## Goal

Rebuild the OCULUS UI as **an illustrated treatise on documents**, using the P(doom) bible's palette, type, grain, crop marks and the spark. Every screen is a plate built from a **P(doom) component, adapted rather than invented**. It stays a Tauri desktop app, keeps all 33 working tools and their engines, and carries Araskova branding.

## Rules carried from the bible and from the user's past feedback

- **Source of truth:** `~/Documents/Projects-Araskova/anvins-aesthetics/docs/TREATMENT.md` and `app/src/engine/palette.ts`.
- **Adapt P(doom) components.** Don't invent new plates. The user rejected custom-built plates ("only take P(doom) components and modify them").
- **Concrete, legible instruments** (odometer, scope, Form 7-B with rubber stamp, Gantt, prompt with token odds, fuse) over abstract 3D or particle fields. Prefer DOM and CSS; three.js only where a 3D object tells the story, and none in this sub-project.
- **UI motion:**
  - one continuous motion
  - no whip-pans
  - no white or impact flashes
  - no heavy shake (at most 1–2% punch-ins)
  - nothing frozen, nothing half-drawn
- **Pages must not look alike.** Each tool family gets its own idiom.
- **Bone paper only where you read a document** (page previews, OCR text). Everywhere else is dark plates.
- **Browsing stays scannable.** No scroll-jacking or pinned scroll scenes in the app.
- **Tone:** dry, deadpan, technical, never begging, no clichés. No emoji, mascots, glowing brains, neon cyberpunk, or stock "AI" imagery.
- **No green** (the user rejected the acid accent), so there's no acid anywhere.

## Reuse source

The Araskova site (`araskova-main-website/Araskova-final/src/components/pdoom/`) already ports these idioms to React by the same team. Port from it, don't rewrite:
- `util.ts`: eases, `prog`, `clamp`, `lerp`, noise.
- `kit.tsx`: `Spark`, `Stamp`, `RollingNumber`, `SectionLabel`, `slam`, `win`, `wdth`.
- `PromptHero.tsx`: prompt and token-odds idiom (its `Popup` distribution).
- `FuseDeploy.tsx`: the fuse (`Fuse`, `burnAt`, and `LoneButton` for ↻ Run again).
- `telemetry/plates.tsx`: Form 7-B, odometer, rubber stamp and scope (`FormPlate`, `OdometerPlate`, `RubberStamp` with the `RubberFilter` ink texture, `TimingPlate`).

Ported files go to `araskova-doccraft/src/pdoom/` and are retokened to the palette below (the site uses Araskova's `#E73F07` tokens; OCULUS uses the bible's).

## 1. Palette (bible `palette.ts`, verbatim)

| token | dark (default) | paper (light theme) | use |
|---|---|---|---|
| `--ink` | `#0A0A0B` | `#EEE9DF` | field |
| `--ink2` | `#151517` | `#E4DED2` | raised panels, instruments |
| `--graphite` | `#5E5B57` | `#B9B2A5` | hairlines, grid, dim strokes only. **Never text on dark** (2.93:1) |
| `--ash` | `#9C978F` (6.82:1) | `#5E5B57` (5.58:1) | secondary text, mono annotations |
| `--bone` | `#EEE9DF` (16.36:1) | `#0A0A0B` | primary text |
| `--signal` | `#FF4D12` (5.96:1) | `#B83204` (4.96:1) | the one hue: spark, fuse, stamps, primary action, live values |
| `--ember` | `#FF8A3D` | `#FF8A3D` | hot cores only (spark centre, fuse tip). Not for text on paper |
| `--blood` | `#C21D0B` | `#C21D0B` | deep shadow of signal (stamp ink edges, charred fuse) |
| `--sheet` | `#EEE9DF` | `#FFFDF8` | document pages, always paper |

- Text on a signal fill is `--ink` (5.96:1).
- Only signal and ember glow (CSS box-shadow bloom). Bone type is always crisp, with no glow, outline or halo.
- **Status is carried without hue.** Success is a signal rubber stamp (`MERGED`, `COMPRESSED`). Failure is a stamp `REJECTED` in signal plus a typed reason. Warnings are an `ash` dashed box with a mono `NOTE`.
- **Texture:**
  - a faint graph-paper grid on the field (`--graphite` at 8% opacity, 24px)
  - film grain: a static SVG `feTurbulence` noise overlay at 4% opacity with `pointer-events: none`, no animation
  - a light vignette

  Reduced-motion and the `paper` theme keep the grain but drop the vignette.
- Radius is 0 everywhere. Hairlines are 1px. There are no drop shadows or gradients except the spark's bloom.
- Theme: `data-theme="dark|paper"` on `<html>`, default `dark`, localStorage key `oculus.theme`. Paper is the treatise's inverted plates (bone paper, ink lines, same orange). Instruments (`.pd-ink`) stay dark in both themes, as on the Araskova site.

## 2. Type (bible)

Bundled locally with `@fontsource`. No Google Fonts requests.
- **Archivo** (variable, `wdth` 62–125, weight 300–900): display and voice. Big, tight, confident. Width and weight are animated for expression: tool names condense on hover, and stamps use `wdth` 75 at weight 900.
- **IBM Plex Mono**: the machine voice. Labels, tool indices, formats, counters, form fields, the prompt, footnotes and readouts. Uppercase with 0.12–0.25em tracking for labels.
- **Cormorant Garamond italic**: rare, for the reading register only. The empty-state epigraphs and the result-sheet signature line.
- **Noto Sans Malayalam**: Malayalam text in previews and OCR output.
- Swiss grid: asymmetric compositions, generous negative space, small mono annotations beside big display type. Kerned text, typographic punctuation (’ “ ” … – — × −) and tabular figures. No glyphs from outside these families: draw symbols such as ⏎ and ↻ as SVG if a font lacks them.

## 3. Brand

- **Araskova mark:** the rounded-apex chevron (from `Araskova-final/public/images/araskova-logo.png`), redrawn as an inline SVG `<AraskovaMark>` in `currentColor`.
- **Wordmark:** `OCULUS` in Archivo 900 at `wdth` 62, with a signal full stop (`OCULUS.`) and `by Araskova` in mono ash. The section-label format from the site: `· 01 — ORGANIZE`.
- **App icon:** the chevron in signal on `#0A0A0B`, with a small bloom. One source SVG `src-tauri/icons/source.svg` regenerates all sizes. **Window title:** `OCULUS — Araskova`.
- **Copy** is deadpan with footnotes. Hero: "Every PDF tool. / None of the uploads." with the footnote "¹ processed on this machine. Network calls: 0." Banned words: "neural", "magic", "AI-powered", "seamless", "unleash".

## 4. The plates (screens and their P(doom) source)

Each screen names the plate it adapts. A screen leans on its one source plate and doesn't mix in idioms from others.

### Frame (all screens): bookend crop marks and the fuse header
- **Crop marks** sit in the window corners and are drawn in graphite. On first launch they fly in from past the edges (as in the bible's bookends), then stay put.
- **Header** (from the Araskova site's header):
  - mark and wordmark on the left
  - the numbered plate nav `01 ORGANIZE · 02 OPTIMIZE · … · 09 OCR & DATA` in mono, which jumps to sections on the index
  - on the right, the status readout `LOCAL · NETWORK CALLS 0` (the 0 is a `RollingNumber` in signal), the theme toggle and settings
  - a 1px fuse line along the header's bottom edge, which burns while any job runs (the spark at its tip)

### Command bar: **`prompt`** (pre-chorus prompt with token odds)
- ⌘K or the header field opens a single prompt field on a dark field. What you type is set in Plex Mono, and above the caret a **next-token distribution** flickers in, built from tool search: each candidate tool gets a bar and its score normalised to a probability. For example, typing `make it smaller` shows `compress 0.61 · optimize images 0.18 · split 0.09 · remove pages 0.05`.
- ↑/↓ re-sample (the chosen candidate lights signal), and **⏎** launches the tool. The launch is a quick collapse of the field into the spark, which travels to the workspace (300ms, no flash).
- **No match:** the distribution shows one bar, `[no tool] 1.00`, and the mono footnote `nothing here takes "…"`.

### Home / index: **catalogue of plates** (the site's `/products` catalogue and the footer's index of plates)
1. **Hero:** Archivo display, the footnote and a hairline.
2. **Drop sheet** (**`open`**, the luminous construction sheet): a dark sheet with a bone construction grid, axes and compass arcs in graphite.
   - **On drop:** the arcs ignite in signal on the drop, and the spark plots each file as a labelled rectangle (name, pages, size in mono) like the plotter pen drawing primitives.
   - **Smart routing:** under the sheet, the tools that accept the dropped files are listed as a token-odds distribution (the same component as the command bar), ranked by usefulness.
   - **Unsupported files:** a file no tool accepts is drawn struck through with the note `nothing here takes .zip`.
3. **Index of plates:** 9 numbered sections (`· 01 — ORGANIZE` … `· 09 — OCR & DATA`), each a row of tool entries.
   - **Entry:** index `T-01`, tool name in Archivo, one deadpan line and a format tag `PDF → DOCX`.
   - **Hover:** the name condenses (`wdth` 100→75), the top hairline burns signal left to right, and the spark sits at its tip.
   - It stays scannable, with no pinning.

### Tool workspace: one shared structure, a different instrument per family
- **Shared structure:**
  - left, the **file stack** as typed manifest lines (mono, reorderable)
  - centre, the **document on bone paper** (pdf.js thumbnails with hairline frames and mono folios, lazy)
  - right, the **options as Form 7-B** (bureau): typewritten fields, checkboxes and selects drawn as form boxes, with the form number set as the tool index (`FORM T-05`)
  - bottom, the **fuse**: the run button is a lit fuse head `RUN ⏎`. While running, the fuse burns across the full width, file names char to ash as the spark passes them, and there's one notch per file or page.
- **Family instruments** (the centre-pane overlay, each adapted from one plate):

  | family | instrument | source plate |
  |---|---|---|
  | 01 Organize (merge, split, remove, extract, organize) | **Gantt strip** of pages: page ranges as bars, cut points as milestone markers (`CUT AFTER p.3`), removed pages struck through in signal | `leftturn` Gantt |
  | 02 Optimize (compress, repair) | **odometer** of the file size rolling down from input to output bytes, plus a `−62%` readout | `ascent` odometer, as used on the site's telemetry desk |
  | 03 Edit (rotate, numbers, watermark) | **construction sheet** overlay on the page: angle callouts (`r = 90°`), folio positions and watermark guides as dimension lines | `open` construction geometry |
  | 04 Security (protect, unlock) | **REDACTED bars** slam over the page preview when protecting and peel back when unlocking | `ilya` (redacted) |
  | 05–08 Convert (from/to PDF, image, other) | **scope** trace: the conversion's progress drawn as an oscilloscope trace on a dark instrument with `IN PDF · OUT DOCX` readouts | `spacetime` scope / site telemetry |
  | 09 OCR & Data | recognised text **typed as tokens**. Low-confidence words appear as **[MASK] blocks** that unmask when confirmed, and hovering a word shows its alternatives as token odds | `loom` ([MASK]) and `prompt` (odds) |

- **Result: bureau receipt and rubber stamp.**
  - **Success:** a Form 7-B receipt types in, with fields `INPUT`, `OUTPUT`, `PAGES`, `SIZE`, `TIME` and `NETWORK CALLS 0`. Then a signal stamp **slams** (`MERGED`, `COMPRESSED −62%`, `PROTECTED`, `RECOGNISED`) with slight rotation, using `Stamp` from the kit. Actions sit below: `Show in folder`, `Open`, `↻ Run again` (the outro's Regenerate button).
  - **Errors:** a `REJECTED` stamp and the engine's reason typed into a `REASON` field, plus "copy details". No modal dialogs.

### Settings: **Form 7-B** (bureau)
A full-page form: `FORM S-01 · PREFERENCES`. Theme (dark/paper checkboxes), OCR languages, default output folder. Saving stamps `FILED`.

### Editor and forms routes
Restyled with the same tokens and kit. No new idioms in this sub-project.

### Removed
`pricing` (the app is free and local). `/convert/*` redirects to `/tool/*`.

## 5. Motion (bible tone, UI-safe)

- **Eases** (from `util.ts`): `outExpo` for slams and entrances, `inOutCubic` for travel. Holds, then snaps. Nothing floats or loops idly, except the spark's flicker while a job runs.
- **Slams** (stamp, receipt fields) use `slam()` from the kit, about 300ms.
- **Forbidden:** whip-pans, white flashes, shake above a 2% punch-in, autoplaying ambient animation on idle screens.
- **`prefers-reduced-motion`:** stamps appear without scale, the fuse becomes a static filled line, and the crop marks are already in place.

## 6. Code structure

- `src/pdoom/`: ported `util.ts`, `kit.tsx` (Spark, Stamp, RollingNumber, SectionLabel, slam, win, wdth), plus new idiom components built only from those:
  - `TokenOdds` (prompt distribution)
  - `ConstructionSheet` (drop sheet)
  - `Form7B` (fields, checkbox, select; plus `Receipt`)
  - `Fuse` (run and progress)
  - `GanttStrip`
  - `Scope`
  - `Redacted`
  - `MaskedText`
  - `CropMarks`
  - `Grain`
- `src/design/`: tokens, theme and contrast helpers.
- `src/components/shell/`: header, command prompt, app frame.
- `src/routes/`: `index.tsx` (catalogue), `tool.$toolId.tsx` (workspace), `settings.tsx`, redirects.
- The tool registry gains `index`, `aliases`, `accepts`, `rank`, `family` (instrument) and `panel` (OCR/extract/dataset). It gets search and routing helpers, feeding both `TokenOdds` uses.
- shadcn primitives: keep only those still needed for behaviour (dialog, select and popover accessibility). Restyle them through tokens to 0 radius, and delete the rest along with the old 2,869-line `styles.css` content.
- Engines (`converter-engine.ts`, `pdf-tools-engine.ts`, `tauri-bridge.ts`) are untouched. No file over about 300 lines.
- Work happens in the `araskova-doccraft` submodule on the branch `feat/araskova-redesign`. The parent repo carries the icons, window title and submodule bump.

## 7. Testing and verification

- `bun run build`, `bun run lint`, `tsc --noEmit` and Vitest pass, and the DavidStack banned patterns hold.
- **Unit:**
  - token contrast pairs (values above)
  - tool search and probabilities (`make it smaller` → compress first)
  - file routing (`.pdf` → merge/compress/ocr; `.zip` → none)
  - theme persistence with storage throwing
  - the options → `ConversionOptions` mapping matches the old page
- **Visual** (browser pane screenshots, examined): home in dark and paper at 1280×850 and 800×600, the prompt with odds, a drop on the construction sheet, and the workspace for one tool per family with its instrument and a stamped receipt. Check against the bible:
  - one hue, no green
  - grain present, bloom only on the spark
  - nothing half-drawn
  - no horizontal scroll at 800px
- **Desktop:** `cargo run` shows the new frame, the icon and the title, and Repair PDF (native) runs to a stamped receipt.

## Out of scope

- New tools (sub-project 3).
- Chaining and Malayalam OCR improvements (sub-project 4).
- three.js scenes, sound design, backend changes, Windows and Linux builds.
