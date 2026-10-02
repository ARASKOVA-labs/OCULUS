# OCULUS Design System and Shell (bible-driven) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the OCULUS UI as an illustrated treatise from adapted P(doom) plates: bible palette and type, prompt-odds search, construction-sheet drop, Form 7-B options, fuse progress, stamped receipts and one instrument per tool family. All 33 tools keep working.

**Architecture:** The UI is the `araskova-doccraft` submodule (TanStack Start, React 19, Tailwind v4).
- `src/pdoom/` holds the P(doom) kit, ported from the Araskova site and retokened to the bible palette, plus the idiom components built from it.
- `src/design/` holds the tokens and theme.
- The tool registry gains search odds and file routing, which drive the prompt and the drop sheet.
- The large route files are split.
- The engines (`converter-engine.ts`, `pdf-tools-engine.ts`, `tauri-bridge.ts`) are not modified.

**Tech Stack:** React 19, TanStack Router/Start, Tailwind v4, pdf.js, `@fontsource` (Archivo, IBM Plex Mono, Cormorant Garamond, Noto Sans Malayalam), Vitest and Testing Library (new), Tauri 2.

**Spec:** `docs/superpowers/specs/2026-10-02-oculus-design-system-and-shell-design.md` (v2). Read it alongside `~/Documents/Projects-Araskova/anvins-aesthetics/docs/TREATMENT.md`.

**Port source:** `SITE=/Users/anvinpshibu/Documents/Projects-Araskova/araskova-main-website/Araskova-final/src/components/pdoom` (read only; never edit it).

## Global Constraints

- **Palette (dark / paper):**

  | token | dark | paper |
  |---|---|---|
  | `--ink` | `#0A0A0B` | `#EEE9DF` |
  | `--ink2` | `#151517` | `#E4DED2` |
  | `--graphite` | `#5E5B57` | `#B9B2A5` |
  | `--ash` | `#9C978F` | `#5E5B57` |
  | `--bone` | `#EEE9DF` | `#0A0A0B` |
  | `--signal` | `#FF4D12` | `#B83204` |
  | `--ember` | `#FF8A3D` | `#FF8A3D` |
  | `--blood` | `#C21D0B` | `#C21D0B` |
  | `--sheet` | `#EEE9DF` | `#FFFDF8` |

  Graphite is never used for text on dark. No other hues anywhere: no green and no acid.
- **Type:** Archivo variable (`wdth` 62–125) for display, IBM Plex Mono for the machine voice, Cormorant Garamond italic only in empty-state epigraphs and the receipt signature, and Noto Sans Malayalam. All bundled, with no `fonts.googleapis.com`.
- **Shape and glow:** radius 0 everywhere, 1px hairlines, no shadows or gradients. Only signal and ember glow.
- **Motion:**
  - eases come from `src/pdoom/util.ts`: `outExpo` for slams, `inOutCubic` for travel
  - forbidden: whip-pans, white flashes, shake above a 2% punch-in, and idle looping animation (the spark flickers only while a job runs or on hover)
  - `prefers-reduced-motion`: stamps without scale, a static fuse, crop marks already in place
- **Paper and theme:** bone `--sheet` only for document pages and OCR text. Theme `data-theme="dark|paper"` on `<html>`, default `dark`, key `oculus.theme`. `.pd-ink` instruments stay dark in both themes.
- **Copy:** deadpan, no emoji, no "neural|magic|AI-powered|seamless|unleash". Wordmark `OCULUS.` (signal full stop) plus `by Araskova`. Window title `OCULUS — Araskova`.
- **Code rules:** DavidStack (no `any`, no `getElementById`, `querySelector` or `createElement`, no `TODO(unresolved)`). New or split files stay under about 300 lines. Engine files are untouched.
- **Where:** all frontend commands run in `araskova-doccraft/` on the branch `feat/araskova-redesign`.

## Review Focus

1. A dropped file no tool accepts (`.zip`) is drawn struck through on the sheet with `nothing here takes .zip`, rather than an empty list. Test in Task 7.
2. An encrypted PDF given to a non-unlock tool gives a `REJECTED` stamp plus the engine reason in the `REASON` field, with no crash and no modal. Test in Task 8.
3. A 500-page PDF mounts ≤ 24 page canvases initially. Test in Task 8.
4. Merge with one file keeps the fuse head unlit, with `Add at least 2 PDFs.` Test in Task 8.
5. localStorage throwing falls back to `dark` without a crash. Test in Task 1.

---

### Task 0: Workspace and test harness

**Files:** `araskova-doccraft/package.json`, create `vitest.config.ts`, `src/test/setup.ts` and `src/test/smoke.test.ts`

- [ ] **Step 1:** From the parent worktree, run `git -c protocol.file.allow=always submodule update --init --reference /Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/araskova-doccraft araskova-doccraft`. Then `cd araskova-doccraft && git switch -c feat/araskova-redesign && bun install`. Expected: `src/` is present.
- [ ] **Step 2:** `bun add -d vitest @testing-library/react @testing-library/user-event @testing-library/jest-dom jsdom`. Add the script `"test": "vitest run"`. The config uses jsdom, setup imports `@testing-library/jest-dom/vitest`, and the alias is `@` → `src`.
- [ ] **Step 3:** `smoke.test.ts` asserts `1 + 1 === 2`. Run `bun run test` → 1 passed. Run `bun run build && bun run lint` → record the baseline result in the commit message.
- [ ] **Step 4:** Commit `chore: add vitest harness`.

### Task 1: Tokens, type, theme and texture

**Files:**
- Create: `src/design/tokens.ts`, `src/design/theme.ts`, `src/design/tokens.test.ts`, `src/design/theme.test.ts`
- Modify: `src/styles.css` (replace lines 1–58 with the new `@theme` and token blocks), `src/routes/__root.tsx:92-104` (drop the Google Fonts links), `package.json`

**Interfaces:**
- Produces:
  - `type TokenName = "ink"|"ink2"|"graphite"|"ash"|"bone"|"signal"|"ember"|"blood"|"sheet"`
  - `TOKENS: Record<"dark"|"paper", Record<TokenName,string>>`
  - `contrastRatio(fg: string, bg: string): number`
  - `type Theme = "dark"|"paper"`
  - `readTheme(): Theme`, `writeTheme(t: Theme): void`, `applyTheme(t: Theme): void` (sets `document.documentElement.dataset.theme`), `useTheme(): [Theme, (t: Theme) => void]`
  - Tailwind utilities `bg-ink bg-ink2 text-bone text-ash text-signal bg-signal border-graphite border-signal bg-sheet text-ember`, fonts `font-display` (Archivo), `font-mono` (Plex Mono), `font-serif` (Cormorant), `font-ml`
  - CSS classes `.pd-ink` (forces dark tokens) and `.pd-grid` (graph-paper field: graphite at 8%, 24px)

- [ ] **Step 1: Failing tests.**
  - `contrastRatio("#FF4D12","#0A0A0B")` ≈ 5.96 (±0.01).
  - On both themes, `bone`, `ash` and `signal` on `ink` are each ≥ 4.5, and `ink` on `signal` is ≥ 4.5.
  - Every `TOKENS.dark` and `TOKENS.paper` value appears in `src/styles.css`.
  - `readTheme()` is `"dark"` when unset, `"paper"` after `writeTheme("paper")`, and `"dark"` when `Storage.prototype.getItem` throws.
  - `writeTheme` doesn't throw when `setItem` throws.
- [ ] **Step 2:** `bun run test` → FAIL (modules missing).
- [ ] **Step 3: Implement.**
  - `tokens.ts` holds the Global Constraints values and the WCAG luminance formula. `theme.ts` uses the key `oculus.theme` with try/catch.
  - `styles.css`:
    - `:root,[data-theme="dark"]` holds the dark values, and `[data-theme="paper"]` holds the paper values
    - `.pd-ink` re-declares the dark values
    - the body is `bg-ink text-bone font-display`, 14px/1.5, `font-variant-numeric: tabular-nums`
    - map the old shadcn variable names (`--background`, `--foreground`, `--primary`, `--border`, `--ring`, `--muted-foreground`, `--card`, `--input`) onto the new tokens so untouched screens still render
    - delete `--success`, `--warning` and `--danger`
  - Fonts: `bun add @fontsource-variable/archivo @fontsource/ibm-plex-mono @fontsource/cormorant-garamond @fontsource-variable/noto-sans-malayalam`. Import `@fontsource-variable/archivo/wdth.css`, Plex Mono 400/500/700, Cormorant 400-italic and 600-italic, and Noto Sans Malayalam in `__root.tsx`.
  - `RootShell` renders `<html lang="en" data-theme="dark">`, and `RootComponent` runs `applyTheme(readTheme())` in a `useEffect`.
- [ ] **Step 4:** `bun run test` → PASS. Then `bun run build && ! grep -r googleapis dist/` → exit 0.
- [ ] **Step 5:** Commit `feat(design): bible palette, local type, dark/paper theme`.

### Task 2: Port the P(doom) kit and the brand

**Files:**
- Create: `src/pdoom/util.ts` (copy of `$SITE/util.ts`), `src/pdoom/kit.tsx`, `src/pdoom/pdoom.css` (imported by `styles.css`), `src/pdoom/kit.test.tsx`, `src/components/brand/araskova-mark.tsx`, `src/components/brand/wordmark.tsx`
- Parent repo: create `src-tauri/icons/source.svg`, regenerate `src-tauri/icons/*`, and set `src-tauri/tauri.conf.json` `app.windows[0].title` to `OCULUS — Araskova`

**Interfaces:**
- Produces:
  - from `util.ts`: `clamp`, `lerp`, `prog`, `ease` (`outExpo`, `outQuart`, `outCubic`, `inOutCubic`, `linear`), `hash`, `noise1`
  - from `kit.tsx`:
    - `slam(t,t0,d?)`, `win(p,a,b)`, `wdth(w: number): CSSProperties` (Archivo: `fontVariationSettings: "wdth" w`, clamped 62–125)
    - `<Spark x y size? />`
    - `<SectionLabel num name note? />` (renders `· 01 — NAME`)
    - `<RollingNumber value from? k tick? />`
    - `<RubberFilter />` (the `#pd-rubber` SVG filter)
    - `<Stamp text sub? k rot? size? />` (merges `$SITE/kit.tsx` `Stamp` with the rubber texture of `$SITE/telemetry/plates.tsx` `RubberStamp`; colour `var(--signal)`)
    - `useAnimatedK(active: boolean, ms: number): number`: a 0→1 rAF ramp, which returns 1 immediately under reduced motion. It's how time-driven slams get their `k` in the app (the site drove them by scroll).
  - `<AraskovaMark size? />`: path `M3 22 L14 4.5 Q16 1.5 18 4.5 L29 22`, viewBox `0 0 32 24`, stroke 4, `currentColor`, `aria-label="Araskova"`
  - `<Wordmark />`: `OCULUS` in Archivo 900 at `wdth` 62, then `.` in `text-signal`, then `by Araskova` in mono ash

- [ ] **Step 1: Failing tests.**
  - `RollingNumber value="380" k={1}` has `aria-label="380"`.
  - `Stamp k={0}` renders nothing. `Stamp text="MERGED" k={1}` renders `MERGED`, with a style `filter` containing `pd-rubber`.
  - `SectionLabel num="01" name="ORGANIZE"` has text `· 01 — ORGANIZE`.
  - `wdth(200).fontVariationSettings === '"wdth" 125'`.
  - `useAnimatedK(true, 300)` returns 1 when `matchMedia("(prefers-reduced-motion: reduce)")` matches.
  - The Wordmark text is `OCULUS.by Araskova` and the `.` has `text-signal`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - Port the kit, replacing `brand-*` classes with token classes and `pd-flex` with `font-display`.
  - `pdoom.css` carries `.pd-spark` and `@keyframes pd-flicker` from `Araskova-final/src/styles.css:1097-1136`, with `--brand-accent` replaced by `--signal`. It also carries `.pd-spark-idle { animation: none }` and the reduced-motion block.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Icon.**
  - `source.svg` is 1024², with an `#0A0A0B` field and the chevron in `#FF4D12` at about 60% width. A soft bloom: a blurred copy at 35% opacity under it.
  - From `araskova-doccraft/`, run `bunx tauri icon ../src-tauri/icons/source.svg -o ../src-tauri/icons`.
  - Open `../src-tauri/icons/128x128.png` with Read and look at it.
- [ ] **Step 6:** Commit in the submodule `feat(pdoom): port kit, brand mark and wordmark`. Commit in the parent `feat(brand): araskova icon and window title`.

### Task 3: Registry odds and routing

**Files:**
- Modify: `src/lib/converter-registry.ts`
- Create: `src/lib/tool-search.ts`, `src/lib/tool-search.test.ts`

**Interfaces:**
- Produces:
  - `ConverterCategory` gains `"ocr-data"`.
  - `ConverterTool` gains:
    - `index: string` (`T-01`…, by `CATEGORY_ORDER` then current order)
    - `aliases: string[]`
    - `accepts: string[]` (lowercase extensions)
    - `rank: number`
    - `family: "organize"|"optimize"|"edit"|"security"|"convert"|"ocr"`
    - `panel?: "ocr"|"extract"|"dataset"`
  - New tools:
    - `ocr` (OCR PDF or image, accepts `pdf png jpg jpeg tif tiff webp`, panel `ocr`)
    - `extract` (Extract structure, accepts `pdf png jpg jpeg`, panel `extract`)
    - `dataset-export` (Export dataset, accepts `txt md json jsonl`, panel `dataset`)
  - `CATEGORY_ORDER: { id: ConverterCategory; num: string; label: string }[]` (`01 ORGANIZE`, `02 OPTIMIZE`, `03 EDIT`, `04 SECURITY`, `05 FROM PDF`, `06 TO PDF`, `07 IMAGE`, `08 OTHER`, `09 OCR & DATA`)
  - `extOf(name: string): string`
  - `searchTools(q: string): ConverterTool[]`
  - `toolOdds(q: string, n = 5): { tool: ConverterTool; p: number }[]` (scores normalised so the returned `p` values sum to 1, rounded to 2 decimals)
  - `toolsForFiles(files: { name: string }[]): ConverterTool[]`
  - `routingOdds(files, n = 6): { tool; p }[]` (p from rank: `1/rank` normalised)

- [ ] **Step 1: Failing tests.**
  - `searchTools("shrink")[0].id === "compress-pdf"`.
  - `toolOdds("make it smaller")[0].tool.id === "compress-pdf"`.
  - `searchTools("combine")[0].id === "merge-pdf"`.
  - `searchTools("docx")` includes `pdf-to-word` and `word-to-pdf`.
  - `searchTools("")` returns all tools in `index` order.
  - `searchTools("zzzz")` is `[]`, and `toolOdds("zzzz")` is `[]`.
  - The sum of `toolOdds("pdf").map(x => x.p)` is within 0.02 of 1.
  - `toolsForFiles([{name:"a.PDF"}])[0].id === "merge-pdf"`; it includes `compress-pdf` and `ocr` and excludes `word-to-pdf`.
  - `toolsForFiles([{name:"a.zip"}])` is `[]`.
  - Files `a.pdf` and `b.png` together include `ocr` and exclude `merge-pdf`.
  - Indices are unique, every tool has at least one alias, and every tool has a `family`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Scoring:** query words are matched case-insensitively against the name, aliases, `fromExt`/`toExt`/`accepts` and the description. Name or alias exact = 3, name or alias substring = 2, format = 1, description word = 0.5. Sum over the query words. Ties go to `rank`.
  - **Minimum aliases:**
    - compress: `shrink smaller reduce size`
    - merge: `combine join`
    - split: `separate cut`
    - ocr: `scan recognise recognize text malayalam`
    - protect: `password encrypt lock`
    - unlock: `decrypt remove password`
    - rotate: `turn`
    - page-numbers: `number folio`
  - **PDF ranks:** merge 1, compress 2, ocr 3, split 4, the rest 10+.
  - **Families:** organize = merge/split/remove/extract/organize. optimize = compress/repair. edit = rotate/page-numbers/watermark. security = protect/unlock. ocr = the 09 tools. Everything else is convert.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit `feat(registry): tool odds, file routing, ocr & data tools`.

### Task 4: Idioms I: token odds, Form 7-B and receipt

**Files:** create `src/pdoom/token-odds.tsx`, `src/pdoom/form7b.tsx`, `src/pdoom/receipt.tsx`, `src/pdoom/idioms1.test.tsx`

**Interfaces:**
- Consumes: the kit (Task 2).
- Produces:
  - `<TokenOdds cands: { label: string; p: number }[] picked?: number onPick?(i: number) header?: string />`
    - adapted from `$SITE/PromptHero.tsx` `Popup` (lines 128–159)
    - the header `p( next | context )` and the state `candidates` or `sampled`
    - rows are `role="option"` with bar width proportional to p
    - the picked row lights signal with `▸`
    - empty `cands` renders one row `[no tool] 1.00`
  - `<Form7B number: string title: string>{children}</Form7B>`, plus `<FormField label value?: string>children</FormField>`, `<FormCheck label checked onChange />` and `<FormSelect label value options onChange />`
    - typewritten fields: Plex Mono and an underline hairline
    - checkboxes are drawn boxes with an `×` mark in signal
    - adapted from `$SITE/telemetry/plates.tsx` `FormPlate` (lines 304–429)
    - native `<input>`/`<select>` underneath, for accessibility
  - `<Receipt fields: [label: string, value: string][] stamp: { text: string; sub?: string } | null tone?: "ok"|"rejected" signature?: string />`
    - fields type in one per 60ms (`useAnimatedK`), then `Stamp` slams
    - `tone="rejected"` stamps `REJECTED`

- [ ] **Step 1: Failing tests.**
  - With cands `[{label:"compress",p:.61},{label:"split",p:.09}]` and `picked={0}`, the TokenOdds shows `0.61`, the first option has `aria-selected="true"` and the header says `sampled`.
  - Empty cands shows `[no tool]`.
  - `FormCheck` toggles through `onChange` on click.
  - `Form7B number="S-01"` renders `FORM S-01`.
  - Receipt (under reduced motion) shows every field label and the stamp text.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement as specified above.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit `feat(pdoom): token odds, Form 7-B, receipt`.

### Task 5: Idioms II: fuse, construction sheet, crop marks and grain

**Files:** create `src/pdoom/fuse.tsx`, `src/pdoom/construction-sheet.tsx`, `src/pdoom/crop-marks.tsx`, `src/pdoom/grain.tsx`, `src/pdoom/idioms2.test.tsx`

**Interfaces:**
- Produces:
  - `<Fuse state: "idle"|"ready"|"burning"|"done" progress?: number notches?: string[] onIgnite?(): void blockedReason?: string | null />`
    - a horizontal braided-cord SVG path, stroked to the burn fraction as in `$SITE/FuseDeploy.tsx` lines 66–100
    - the burnt stretch is graphite/blood, the unburnt cord is ash, and the head is `Spark`
    - `ready` shows the head as a button `RUN ⏎` (`role="button"`, signal fill, ink text)
    - `burning` with no progress travels indeterminately (`inOutCubic`, 2.4s)
    - notch labels char (opacity 0.3 and graphite) once the burn passes them
    - when `blockedReason` is set, the head is unlit and the reason is printed beside it in mono ash
  - `<ConstructionSheet files: { name: string; pages?: number; size: number; rejected?: boolean }[] onFiles(files: File[]): void />`
    - a `.pd-ink` sheet with a bone construction grid, axes and compass arcs (graphite SVG)
    - on drag-over the arcs turn signal
    - each file is plotted as a labelled rectangle (stroke dash-offset draw, 400ms)
    - rejected files are struck through with `nothing here takes .{ext}`
    - includes a `Browse` button: under Tauri it calls `selectNativeFiles()` and turns each path into `new File([await readNativeBinaryFile(path)], basename)`; otherwise a hidden `<input type=file multiple>` triggered by a ref
  - `<CropMarks fly?: boolean />`: four graphite L marks (14px arms) at the corners of the positioned parent. With `fly`, they enter from 40px outside over 600ms `outExpo`.
  - `<Grain />`: a fixed full-window SVG `feTurbulence` (baseFrequency 0.9) at 4% opacity, `pointer-events-none`, static. It also adds a radial vignette (ink at 35% at the corners) on the dark theme only.

- [ ] **Step 1: Failing tests.**
  - `Fuse state="ready"` has a button `RUN ⏎`, and clicking it calls `onIgnite`.
  - `blockedReason="Add at least 2 PDFs."` disables the button and shows the text.
  - `state="burning" progress={0.5}` has `role="progressbar"` with `aria-valuenow="50"`.
  - ConstructionSheet with a rejected `a.zip` shows `nothing here takes .zip`.
  - Firing a `drop` with a PDF calls `onFiles`.
  - CropMarks renders 4 `[data-crop]`.
  - Grain renders a `[data-vignette]` only when `data-theme` is `dark`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit `feat(pdoom): fuse, construction sheet, crop marks, grain`.

### Task 6: App frame, header and command prompt

**Files:**
- Create: `src/components/shell/app-frame.tsx`, `src/components/shell/header.tsx`, `src/components/shell/command-prompt.tsx`, `src/lib/job-activity.ts`, `src/components/shell/shell.test.tsx`
- Modify: `src/routes/__root.tsx` (use `AppFrame` instead of `OculusShell`)

**Interfaces:**
- Consumes: `toolOdds` (Task 3), `TokenOdds`, `CropMarks`, `Grain`, `Fuse`, `RollingNumber`, `Wordmark`, `AraskovaMark` and `useTheme`.
- Produces:
  - `<AppFrame>{children}</AppFrame>`: `.pd-grid` field, `Grain`, and `CropMarks fly` on first mount per session (`sessionStorage` key `oculus.framed`, with try/catch)
  - `<Header onOpenPrompt() />`
  - `<CommandPrompt open onOpenChange />`
  - `setJobActive(active: boolean): void` and `useJobActive(): boolean`, a tiny store. The header fuse burns while it's true.

- [ ] **Step 1: Failing tests** (memory router):
  - The header shows the plate nav items `01 ORGANIZE` … `09 OCR & DATA` (as links to `/#plate-01` …) and the readout text `NETWORK CALLS`.
  - The theme toggle has `aria-label="Switch to paper theme"`.
  - `Meta+k` opens the prompt (`role="dialog"`).
  - Typing `make it smaller` shows the odds with `compress` first.
  - `ArrowDown` moves `aria-selected` to the second option.
  - `Enter` navigates to `/tool/<selected id>`.
  - `Escape` closes the prompt and focus returns to the trigger.
  - `zzzz` shows `[no tool]` and `nothing here takes "zzzz"`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Header:** 52px, with a graphite hairline bottom that becomes the `Fuse` (indeterminate) when `useJobActive()` is true.
    - left: mark and wordmark
    - centre: the plate nav, mono 10px with 0.22em tracking
    - right: `LOCAL · NETWORK CALLS` plus `RollingNumber value="0"` in signal, the theme toggle, and a settings link
    - the prompt trigger is a mono field `> what should happen to your file` with a `⌘K` hint
  - **Prompt:** a Radix dialog on `bg-ink/95` with a single Plex Mono input and `TokenOdds` above the caret, fed by `toolOdds`. On launch, the spark travels from the caret to the top-left over 300ms and then the app navigates. There's no flash.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit `feat(shell): treatise frame, fuse header, prompt with token odds`.

### Task 7: Home: catalogue of plates

**Files:**
- Create: `src/lib/pending-files.ts` (+ test), `src/components/home/hero.tsx`, `src/components/home/drop-plate.tsx`, `src/components/home/plate-index.tsx`, `src/components/home/tool-entry.tsx`, `src/components/home/home.test.tsx`
- Modify: `src/routes/index.tsx`. Move the old sections (`#files #ocr #extract #export #settings`, lines 346–845) unchanged into `src/components/legacy-home-sections.tsx` for Task 10, and render only the new home.

**Interfaces:**
- Consumes: `ConstructionSheet`, `TokenOdds`, `SectionLabel`, `routingOdds`, `toolsForFiles`, `CATEGORY_ORDER`.
- Produces: `setPendingFiles(files: File[]): void`, `takePendingFiles(): File[]` (returns the files and clears them).

- [ ] **Step 1: Failing tests.**
  - `takePendingFiles` returns the set files, then `[]` on the next call.
  - Home renders `Every PDF tool.`, `None of the uploads.` and `¹ processed on this machine. Network calls: 0.`
  - It renders 9 section labels `· 01 — ORGANIZE` … `· 09 — OCR & DATA`, with the ids `plate-01`…`plate-09`.
  - Dropping `a.pdf` shows a `TokenOdds` headed `p( tool | a.pdf )` with `merge` first.
  - Dropping `a.zip` shows `nothing here takes .zip` and no odds.
  - Clicking an odds row calls `setPendingFiles` and navigates to `/tool/<id>`.
  - Each tool entry shows its `index`, name and `PDF → DOCX`-style tag.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Hero:** Archivo 900 at `wdth` 62, `clamp(48px, 7vw, 112px)`, tight leading, with the footnote in mono ash. Asymmetric: the type sits left, and the drop plate takes the right 5/12 at ≥1100px or stacks below that.
  - **Tool entry:** a hairline top border. Hover changes the name's `wdth` from 100 to 75 (200ms `outExpo`). A signal line grows along the top border left to right, with `Spark` at its tip. The index is set in mono.
  - **Layout:** 4 columns, or 2 below 1100px. No pinning or scroll effects.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit `feat(home): catalogue of plates with construction-sheet drop`.

### Task 8: Tool workspace: manifest, sheet, Form 7-B, fuse and receipt

**Files:**
- Create:
  - `src/routes/tool.$toolId.tsx`
  - `src/components/workspace/manifest.tsx`
  - `src/components/workspace/page-sheet.tsx`
  - `src/components/workspace/tool-form.tsx` (port of the option controls in `convert.$toolId.tsx`, onto `Form7B`)
  - `src/components/workspace/options-state.ts` (+ test)
  - `src/components/workspace/result.tsx`
  - `src/components/workspace/instrument.tsx`
  - `src/components/workspace/workspace.test.tsx`
- Modify: `src/routes/convert.$toolId.tsx` and `src/routes/convert.index.tsx` become redirects to `/tool/$toolId` and `/`.

**Interfaces:**
- Consumes:
  - `executeConversion(toolId: string, file: File, options: ConversionOptions): Promise<ConversionResult>` and `renderUniversalPdfPage` (`converter-engine.ts`)
  - `selectNativeFiles`, `showInNativeFolder`, `isTauri` (`tauri-bridge.ts`)
  - `takePendingFiles`, `setJobActive`, `Fuse`, `Receipt`, `Form7B`
- Produces:
  - `type ToolOptionsState` (the state at `convert.$toolId.tsx:79-95` as one object)
  - `defaultOptions(): ToolOptionsState`
  - `toConversionOptions(s, extra: { additionalFiles: File[]; nativePath?: string }): ConversionOptions` (the mapping at `convert.$toolId.tsx:224-242`, with `parseNumbersList` moved here)
  - `runBlockReason(toolId: string, files: File[]): string | null`
  - `resultFields(r: ConversionResult, input: File[]): [string,string][]` → `INPUT`, `OUTPUT`, `PAGES`, `SIZE` (`1.0 KB → 380 B · −62%`), `TIME`, `NETWORK CALLS 0`
  - `stampFor(toolId: string, r: ConversionResult): { text: string; sub?: string }`, for example merge → `MERGED`, compress → `COMPRESSED` with sub `−62%`, protect → `PROTECTED`, unlock → `UNLOCKED`, ocr → `RECOGNISED`, convert → `CONVERTED`, failure → `REJECTED`
  - `<Instrument family: ConverterTool["family"] state: WorkspaceRunState />`: placeholder slot, filled in Task 9
  - `type WorkspaceRunState = { files: File[]; options: ToolOptionsState; running: boolean; result: ConversionResult | null }`

- [ ] **Step 1: Failing tests.**
  - `toConversionOptions(defaultOptions(), {additionalFiles: []})` deep-equals the old page's initial build (`pageOrder: [1,2]`, `pagesToRemove: [1]`, `compressionLevel: "recommended"`, `watermarkText: "CONFIDENTIAL"`, `rotationDegrees: 90`, `docxMode: "hybrid"`, `language: "eng"` …).
  - `runBlockReason("merge-pdf", [one])` is `"Add at least 2 PDFs."`, and `null` with two.
  - `runBlockReason("compress-pdf", [])` is `"Add a file to start."`
  - `resultFields` with input 1000 B and output 380 B includes `["SIZE", "1.0 KB → 380 B · −62%"]`.
  - `stampFor("compress-pdf", ok)` is `{text:"COMPRESSED", sub:"−62%"}`.
  - The workspace with an `executeConversion` mock resolving `{success:false, error:"Encrypted PDF: password required"}` shows `REJECTED` and `Encrypted PDF: password required`, with no `role="alertdialog"`.
  - `PageSheet` with a mocked 500-page doc mounts ≤ 24 `canvas`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Layout:** a `240px | 1fr | 340px` grid with the `Fuse` full-width at the bottom.
    - **Manifest:** typed lines `01  report.pdf   12 pp   1.2 MB`, reordered by drag (HTML5 DnD), with remove (`×`) and `+ add`.
    - **Page sheet:** pages on `bg-sheet` with hairline frames and mono folios, rendered lazily with IntersectionObserver, with `<Instrument>` overlaid.
    - **Form:** `Form7B number={tool.index} title={tool.name}`.
  - **Running:** the fuse ignites on click or ⏎ (when not in a text field). It runs `setJobActive(true)` and `executeConversion`, and the result becomes a `Receipt` with `resultFields`, `stampFor` and the signature `filed by OCULUS · Araskova` in Cormorant italic.
  - **Actions:** `Show in folder` (Tauri), `Open`/`Download` and `↻ Run again`.
  - **Pending files:** call `takePendingFiles()` on mount.
  - **Panel tools:** tools with a `panel` render the Task 10 panel in place of the form and sheet. Until then, show a dashed ash `NOTE: arrives in this release.`
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Browser.** With the dev server up:
  - Merge `test_samples/sample_digital.pdf` with itself, and compress it.
  - Screenshot the burning fuse and both stamped receipts, and look at them.
  - Check for nothing half-drawn and no colour but orange.
- [ ] **Step 6:** Commit `feat(workspace): manifest, page sheet, Form 7-B, fuse, stamped receipt`.

### Task 9: Family instruments (organize, optimize, edit, security, convert)

**Files:** create `src/pdoom/instruments/gantt-strip.tsx`, `odometer.tsx`, `construction-overlay.tsx`, `redacted.tsx`, `scope.tsx` and `instruments.test.tsx`. Modify `src/components/workspace/instrument.tsx` to dispatch on `family`.

**Interfaces:**
- Consumes: `WorkspaceRunState` (Task 8), `parsePageRanges(expr, max)` (`pdf-tools-engine.ts`) and the kit.
- Produces:
  - `<GanttStrip pageCount: number ranges?: [number,number][] cuts?: number[] removed?: number[] />`
    - adapted from `leftturn` Gantt, as in `$SITE/telemetry/plates.tsx` `TimingPlate`
    - pages are cells on one strip, ranges are bars, and cuts are milestone diamonds labelled `CUT AFTER p.N`
    - removed pages are struck in signal
  - `<Odometer from: number to: number k: number />`: bytes as `RollingNumber` drums (`$SITE/telemetry/plates.tsx` `OdometerPlate`) plus a `−62%` readout
  - `<ConstructionOverlay rotation?: 90|180|270 folio?: PageNumberPosition watermark?: { text: string; angle: number } />`: graphite dimension lines and callouts (`r = 90°`, the folio position box, the watermark baseline at its angle)
  - `<Redacted k: number mode: "protect"|"unlock" />`: ink bars slam over the page sheet (protect) or peel off it (unlock), with `REDACTED` set in mono on the bars
  - `<Scope k: number from: string to: string running: boolean />`: a `.pd-ink` CRT panel with a phosphor trace (signal, with echo copies at 30/15% opacity) and readouts `IN PDF · OUT DOCX`

  Mapping:

  | family | instrument |
  |---|---|
  | organize | `GanttStrip` (ranges and cuts from `splitRange`, `removed` from `pagesToRemoveStr`, `extract` highlights) |
  | optimize | `Odometer` (shown once a result exists; compress and repair) |
  | edit | `ConstructionOverlay` |
  | security | `Redacted` |
  | convert | `Scope` |
  | ocr | handled by Task 10 |

- [ ] **Step 1: Failing tests.**
  - `GanttStrip pageCount={10} cuts={[3]}` shows `CUT AFTER p.3` and 10 `[data-page]` cells. With `removed={[2]}`, cell 2 has `data-removed`.
  - `Odometer from={1000} to={380} k={1}` has the label `380` and shows `−62%`.
  - `ConstructionOverlay rotation={90}` shows `r = 90°`.
  - `Redacted mode="protect" k={1}` renders ≥ 3 `[data-bar]`.
  - `Scope from="PDF" to="DOCX"` shows `IN PDF` and `OUT DOCX`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run → PASS. Browser: screenshot one tool per family (split, compress, rotate, protect, pdf-to-word) and look at each one. The five must not look alike.
- [ ] **Step 5:** Commit `feat(instruments): gantt, odometer, construction, redacted, scope`.

### Task 10: OCR & Data panels and the settings form

**Files:**
- Create: `src/pdoom/instruments/masked-text.tsx`, `src/components/panels/ocr-panel.tsx`, `src/components/panels/extract-panel.tsx`, `src/components/panels/dataset-panel.tsx`, `src/components/panels/panels.test.tsx`
- Modify: `src/routes/settings.tsx` (a real page), and `src/routes/ocr.tsx`, `extract.tsx` and `datasets.tsx` (redirect to `/tool/ocr`, `/tool/extract` and `/tool/dataset-export`)
- Delete: `src/components/legacy-home-sections.tsx`

**Interfaces:**
- Consumes: `useOculus()` (unchanged), the JSX and handlers from `legacy-home-sections.tsx`, `TokenOdds`, `Form7B`, `Receipt` and `useTheme`.
- Produces:
  - `<MaskedText blocks: { text: string; confidence: number }[] threshold?: number /* default 0.80 */ onConfirm?(i: number) />`
    - adapted from `loom`'s [MASK]
    - words at or above the threshold are typed as tokens in `font-ml` or display type
    - words below it render as solid `[MASK]` blocks (bone-on-graphite) showing the word on hover, with a `TokenOdds` of `[{label: word, p: confidence}, {label:"[unclear]", p: 1-confidence}]`
    - clicking confirms the word and unmasks it
  - `<OcrPanel />`, `<ExtractPanel />`, `<DatasetPanel />`: the old sections moved as is, restyled with Form 7-B controls and the `Receipt` with stamps `RECOGNISED`/`EXTRACTED`/`EXPORTED`. OcrPanel renders its text through `MaskedText`.
  - `/settings`: `Form7B number="S-01" title="PREFERENCES"` with:
    - theme (`FormCheck` Dark/Paper)
    - OCR languages (English/Malayalam)
    - default output folder

    Saving slams a `FILED` stamp.

- [ ] **Step 1: Failing tests.**
  - `MaskedText` with `[{text:"നമസ്കാരം",confidence:.95},{text:"ലോകം",confidence:.4}]` shows `നമസ്കാരം` and one `[MASK]`. Clicking the mask reveals `ലോകം`.
  - `/tool/ocr` renders the OCR form with a language select containing `Malayalam`.
  - `/settings`: choosing Paper sets `document.documentElement.dataset.theme === "paper"`, and saving shows `FILED`.
  - `/ocr` redirects to `/tool/ocr`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement. The text area is the only bone-paper surface besides page previews.
- [ ] **Step 4:** Run → PASS. Desktop: start `cargo run` with the dev server up, OCR `test_samples/sample_mal.png`, screenshot the masked text and stamped receipt, and look at them.
- [ ] **Step 5:** Commit `feat(ocr): [MASK] text, ocr/extract/dataset panels, settings form`.

### Task 11: Restyle the remainder and delete the old UI

**Files:**
- Modify: `src/routes/editor.tsx`, `src/routes/forms.tsx`, and the 404 and error components in `src/routes/__root.tsx` (the 404 is a `REJECTED` stamp with `no such plate`)
- Delete:
  - `src/routes/pricing.tsx`
  - `src/components/oculus-shell.tsx`
  - `src/components/work-progress-bar.tsx` (replaced by `Fuse`) and its imports
  - `src/components/ui/{carousel,calendar,chart,menubar,navigation-menu,pagination,breadcrumb,context-menu,sidebar}.tsx`
  - their now-unused deps (`embla-carousel-react`, `react-day-picker`, `recharts`, if present)
- Modify: `src/styles.css`. Delete all old selectors (`timeline-*`, old shell). Target under 600 lines.

- [ ] **Step 1: Failing test.**
  - `/pricing` renders `no such plate`.
  - Rendered text on `/`, `/tool/merge-pdf`, `/settings`, `/editor` and `/forms` matches neither `/neural|magic|AI-powered|seamless|unleash/i` nor an emoji (`/\p{Extended_Pictographic}/u`).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Restyle and delete.
  - `grep -rn "oculus-shell\|pricing\|timeline-\|work-progress-bar" src` → no matches.
  - `grep -rniE "#(16a34a|22c55e|d97706|dc2626|3b82f6)|green|emerald|blue-|rounded-(md|lg|full|xl)" src` → no matches (radius 0, one hue).
  - `wc -l src/styles.css` < 600.
- [ ] **Step 4:** `bun run test && bun run lint && bunx tsc --noEmit && bun run build` → all exit 0.
- [ ] **Step 5:** Commit `refactor: restyle editor/forms, remove pricing and the old UI`.

### Task 12: Verify against the bible and integrate

- [ ] **Step 1:** With the dev server in the browser pane, screenshot and look at each of:
  - `/` in dark and paper at 1280×850 and 800×600
  - the prompt with `make it smaller`
  - a drop of `sample_digital.pdf` and of a `.zip`
  - one workspace per family with its stamped receipt
  - `/settings`

  Check each against `TREATMENT.md`:
  - one hue, no green
  - grain present, bloom only on the spark
  - crisp bone type with no glow
  - nothing half-drawn
  - no horizontal scroll at 800px
  - pages not alike

  Fix any miss in the owning component.
- [ ] **Step 2:** Tauri: from the parent, `cd src-tauri && cargo run` with the dev server up. Check the title `OCULUS — Araskova`, the new icon in the Dock, the crop marks flying in on launch, and Repair PDF (native) reaching a stamped receipt. `cargo test` still passes 36.
- [ ] **Step 3:** In the parent: `git add araskova-doccraft && git commit -m "chore: bump araskova-doccraft to the redesign"`. Push nothing unless the user approves.
