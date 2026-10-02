# OCULUS Design System and App Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restyle the OCULUS UI in Anvin's aesthetic with Araskova branding: tokens, fonts, the shell, the command palette, the smart-routing home and one tool workspace. All 33 tools keep working.

**Architecture:** The UI is the `araskova-doccraft` submodule (TanStack Start + React 19 + Tailwind v4 + shadcn). Design tokens live in `src/design/tokens.ts`, mirrored into `src/styles.css`. The tool registry gains search and routing metadata, which feeds the palette and the drop zone. The giant `convert.$toolId.tsx`, `index.tsx` and `oculus-shell.tsx` are split into focused components. The engines (`converter-engine.ts`, `pdf-tools-engine.ts`, `tauri-bridge.ts`) are not modified.

**Tech Stack:** React 19, TanStack Router/Start, Tailwind v4, Radix/shadcn, `cmdk` (already used by `ui/command.tsx`), pdf.js, `@fontsource`, Vitest + Testing Library (new), Tauri 2.

**Spec:** `docs/superpowers/specs/2026-10-02-oculus-design-system-and-shell-design.md`

## Global Constraints

- One hue: `--signal` `#E73F07` (dark) / `#C2340A` (paper). No green, blue, red or yellow anywhere, including status states.
- Tokens are exactly the spec §1 table. Radius 0 on frames and panels, 2px on inputs and buttons. No shadows, gradients, pills or blur. Only signal may glow.
- Fonts are bundled locally: Roboto Flex (variable), Space Mono 400/700, Noto Sans Malayalam (variable). No requests to fonts.googleapis.com.
- Theme: `data-theme="dark|paper"` on `<html>`, default `dark`, persisted under the localStorage key `oculus.theme`.
- Copy: no "neural", "magic", "AI-powered", "seamless", "unleash", and no emoji. Wordmark `OCULUS.` with a signal full stop, plus `by Araskova`. Window title `OCULUS — Araskova`.
- Motion: outExpo `cubic-bezier(.16,1,.3,1)`, travel `cubic-bezier(.65,0,.35,1)`. `prefers-reduced-motion` disables all animation.
- DavidStack: no `any`, no `getElementById`/`querySelector`/`createElement`, no `TODO(unresolved)`.
- No file over about 300 lines among new or split files. Engine files are untouched.
- All frontend commands run in `araskova-doccraft/` on the branch `feat/araskova-redesign`.

## Review Focus

1. A dropped file that no tool accepts (e.g. `.zip`) shows "No tool takes .zip files." rather than an empty grid. Test in Task 5.
2. An encrypted PDF given to a tool that can't open it shows an inline `ERR` block with the engine message. The app doesn't crash or open a modal. Test in Task 6.
3. A 500-page PDF in the preview renders thumbnails lazily, with ≤ 24 canvases mounted initially. Test in Task 6.
4. Merge PDF with a single file keeps `RUN` disabled with the note "Add at least 2 PDFs." Test in Task 6.
5. localStorage throwing (private mode, blocked) falls back to `dark` without crashing. Test in Task 1.

---

### Task 0: Workspace setup and test harness

**Files:**
- Modify: `araskova-doccraft/package.json` (devDeps, `test` script)
- Create: `araskova-doccraft/vitest.config.ts`, `araskova-doccraft/src/test/setup.ts`, `araskova-doccraft/src/test/smoke.test.ts`

- [ ] **Step 1:** In the parent worktree, run `git -c protocol.file.allow=always submodule update --init --reference /Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/araskova-doccraft araskova-doccraft`, then in `araskova-doccraft/`: `git switch -c feat/araskova-redesign && bun install`. Expected: `src/` is present and the branch is created.
- [ ] **Step 2:** `bun add -d vitest @testing-library/react @testing-library/user-event @testing-library/jest-dom jsdom`. Add the script `"test": "vitest run"`. `vitest.config.ts` uses `environment: "jsdom"`, `setupFiles: ["src/test/setup.ts"]` (imports `@testing-library/jest-dom/vitest`), and the `@` alias to `src`.
- [ ] **Step 3:** Write `smoke.test.ts`: `expect(1 + 1).toBe(2)`. Run `bun run test`. Expected: 1 passed.
- [ ] **Step 4:** Baseline: `bun run build && bun run lint`. Expected: both exit 0 (note any pre-existing lint errors in the commit message instead of fixing them here).
- [ ] **Step 5:** Commit: `chore: add vitest harness`.

### Task 1: Tokens, fonts and theme

**Files:**
- Create: `src/design/tokens.ts`, `src/design/theme.ts`, `src/design/tokens.test.ts`, `src/design/theme.test.ts`
- Modify: `src/styles.css` (replace the `@theme inline` and `:root` blocks at lines 1–58 and add `[data-theme="paper"]`), `src/routes/__root.tsx:92-104` (drop the Google Fonts links, import fonts), `package.json`

**Interfaces:**
- Produces: `TOKENS: Record<"dark"|"paper", Record<TokenName, string>>`, `type TokenName = "field"|"panel"|"rule"|"rule-strong"|"ash"|"type"|"signal"|"signal-ink"|"paper"`, `contrastRatio(fg: string, bg: string): number`, `type Theme = "dark"|"paper"`, `readTheme(): Theme`, `writeTheme(t: Theme): void`, `applyTheme(t: Theme): void` (sets `document.documentElement.dataset.theme`), `useTheme(): [Theme, (t: Theme) => void]`.
- Tailwind colour utilities: `bg-field bg-panel border-rule border-rule-strong text-ash text-type text-signal bg-signal text-signal-ink bg-paper`. Fonts: `font-display` (Roboto Flex), `font-mono` (Space Mono), `font-ml` (Noto Sans Malayalam).

- [ ] **Step 1: Failing tests.** `tokens.test.ts`:
  - `contrastRatio("#E73F07","#0A0A0A")` ≈ 4.82 (±0.01).
  - For both themes, each of `type`, `ash` and `signal` on `field`, plus `signal-ink` on `signal`, is ≥ 4.5 (measured minimum: 4.82).
  - Every `TOKENS.dark` value appears in `styles.css` (read with `fs.readFileSync`).

  `theme.test.ts`:
  - `readTheme()` returns `"dark"` when the key is unset.
  - It returns `"paper"` after `writeTheme("paper")`.
  - It returns `"dark"` when `localStorage.getItem` throws (stub with `vi.spyOn(Storage.prototype,"getItem").mockImplementation(() => { throw new Error() })`).
  - `writeTheme` does not throw when `setItem` throws.
- [ ] **Step 2:** `bun run test`. Expected: FAIL, modules not found.
- [ ] **Step 3: Implement.**
  - `tokens.ts` holds the spec §1 values verbatim and the WCAG relative-luminance formula.
  - `theme.ts` uses the key `oculus.theme` with try/catch around storage.
  - Rewrite the head of `styles.css`: `@theme inline` maps `--color-field: var(--field)` etc. and `--font-display/--font-mono/--font-ml`. `:root, [data-theme="dark"]` holds the dark values and `[data-theme="paper"]` the paper values. The body uses `bg-field text-type font-display`, 14px/1.5, `font-variant-numeric: tabular-nums`. Remove the old `--success/--warning/--danger`, and map the old shadcn names (`--background`, `--foreground`, `--primary`, `--border`, `--ring`, …) onto the new tokens so untouched components still render.
  - `bun add @fontsource-variable/roboto-flex @fontsource/space-mono @fontsource-variable/noto-sans-malayalam`. Import `@fontsource-variable/roboto-flex/full.css` (the full axes, including `wdth`), `@fontsource/space-mono/400.css`, `@fontsource/space-mono/700.css` and `@fontsource-variable/noto-sans-malayalam` in `__root.tsx`. Delete the three googleapis link entries.
  - `RootShell` renders `<html lang="en" data-theme="dark">`, and `RootComponent` calls `applyTheme(readTheme())` in a `useEffect`.
- [ ] **Step 4:** `bun run test`. Expected: PASS. Also run `bun run build` and `grep -r googleapis dist/` → no matches.
- [ ] **Step 5:** Commit: `feat(design): araskova tokens, local fonts, dark/paper theme`.

### Task 2: Brand and status primitives

**Files:**
- Create: `src/components/brand/araskova-mark.tsx`, `src/components/brand/wordmark.tsx`, `src/components/status/status-tag.tsx`, `src/components/status/spark-progress.tsx`, `src/components/brand/brand.test.tsx`
- Parent repo: create `src-tauri/icons/source.svg`. Regenerate `src-tauri/icons/*`. Modify `src-tauri/tauri.conf.json` (`app.windows[0].title` → `OCULUS — Araskova`).

**Interfaces:**
- Produces:
  - `<AraskovaMark size?: number className?: string />`: the stroked chevron, `currentColor`, `aria-label="Araskova"`.
  - `<Wordmark />`: renders `OCULUS` and a `.` in `text-signal`, plus `by Araskova` in `text-ash`.
  - `<StatusTag kind: "done"|"err"|"warn"|"run" children?: ReactNode />`: labels `DONE ✓` / `ERR` / `WARN` / `RUN`.
  - `<SparkProgress value?: number /* 0..1, undefined = indeterminate */ ticks?: number label?: string />` with `role="progressbar"` and `aria-valuenow` when determinate.

- [ ] **Step 1: Failing tests.**
  - The Wordmark text content is `OCULUS.by Araskova` and the `.` has class `text-signal`.
  - StatusTag `err` renders `ERR`.
  - SparkProgress `value={0.4}` has `aria-valuenow="40"`. With no value it has no `aria-valuenow`.
  - With `ticks={5}` it renders 5 tick elements (`data-tick`).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Chevron SVG path:** the rounded-apex Λ from `Araskova-final/public/images/araskova-logo.png`. viewBox `0 0 32 24`, path `M3 22 L14 4.5 Q16 1.5 18 4.5 L29 22`, stroke-width 4, `stroke-linecap="butt"`, `fill="none"`.
  - **SparkProgress:** a 1px `--rule` track and a `--signal` lit segment. The spark is a 7px `#fff3ea` dot with the bloom `0 0 6px 2px var(--signal), 0 0 22px 6px rgb(231 63 7 / .55)`. Indeterminate mode runs a 2.4s `cubic-bezier(.65,0,.35,1)` loop, and reduced-motion shows a static 40% line.
  - **StatusTag:** Space Mono uppercase, tracking 0.12em. `err` has a 1px signal border, `warn` a dashed rule border.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Icon.** `source.svg` is 1024×1024: an `#0A0A0A` square with the chevron scaled to about 60% width in `#E73F07`. From `araskova-doccraft/`, run `bunx tauri icon ../src-tauri/icons/source.svg -o ../src-tauri/icons`. Expected: png/icns/ico rewritten. Open `128x128.png` with the Read tool to check it visually.
- [ ] **Step 6:** Commit in the submodule (`feat(brand): mark, wordmark, status tag, spark progress`) and in the parent (`feat(brand): araskova app icon and window title`).

### Task 3: Registry search and routing metadata

**Files:**
- Modify: `src/lib/converter-registry.ts` (type, categories, 3 new entries, metadata on all 33)
- Create: `src/lib/tool-search.ts`, `src/lib/tool-search.test.ts`

**Interfaces:**
- Produces:
  - `ConverterCategory` gains `"ocr-data"`.
  - `ConverterTool` gains `index: string` (`"T-01"`…, ordered by category then current order), `aliases: string[]`, `accepts: string[]` (lowercase extensions without the dot, e.g. `["pdf"]`), `rank: number` (lower = more useful) and `panel?: "ocr"|"extract"|"dataset"`.
  - New tools:
    - `ocr` (`OCR PDF or image`, accepts `pdf png jpg jpeg tif tiff webp`, panel `ocr`)
    - `extract` (`Extract structure`, panel `extract`)
    - `dataset-export` (`Export dataset`, accepts `txt md json jsonl`, panel `dataset`; it reads the current OCR result from `useOculus()`)
  - `CATEGORY_ORDER: { id: ConverterCategory; num: string; label: string }[]` (`01 ORGANIZE` … `09 OCR & DATA`).
  - `searchTools(query: string): ConverterTool[]`
  - `toolsForFiles(files: { name: string }[]): ConverterTool[]`
  - `extOf(name: string): string`

- [ ] **Step 1: Failing tests.**
  - `searchTools("shrink")[0].id === "compress-pdf"`.
  - `searchTools("combine")[0].id === "merge-pdf"`.
  - `searchTools("docx")` contains `pdf-to-word` and `word-to-pdf`.
  - `searchTools("")` returns all tools in `index` order.
  - `searchTools("zzzz")` is `[]`.
  - `toolsForFiles([{name:"a.PDF"}])[0].id === "merge-pdf"` and includes `compress-pdf` and `ocr`, but excludes `word-to-pdf`.
  - `toolsForFiles([{name:"a.zip"}])` is `[]`.
  - Mixed input `[{name:"a.pdf"},{name:"b.png"}]` returns only tools accepting both (includes `ocr`).
  - Every tool has a unique `index` and `aliases.length ≥ 1`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Search:** case-insensitive scoring. Exact name prefix = 3, alias exact = 3, alias or name substring = 2, format (`fromExt`/`toExt`/`accepts`) match = 1. Ties go to `rank`. Drop zero scores.
  - **`toolsForFiles`:** the intersection of `accepts` over the files' extensions, sorted by `rank`. PDF ranks: merge 1, compress 2, ocr 3, split 4, then the rest at 10+.
  - **Aliases at minimum:** compress `shrink, reduce, smaller`; merge `combine, join`; split `separate, cut`; ocr `scan, recognise, text`; protect `password, encrypt`; unlock `decrypt, remove password`.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit: `feat(registry): tool search, file routing, ocr & data tools`.

### Task 4: Top bar and command palette

**Files:**
- Create: `src/components/shell/top-bar.tsx`, `src/components/shell/command-palette.tsx`, `src/components/shell/app-shell.tsx`, `src/components/shell/shell.test.tsx`
- Modify: `src/routes/__root.tsx` (use `AppShell`)
- Delete: `src/components/oculus-shell.tsx` (in Task 8, after nothing imports it)

**Interfaces:**
- Consumes: `searchTools` and `Wordmark`/`AraskovaMark` from Tasks 2–3, plus `useTheme`.
- Produces:
  - `<AppShell>{children}</AppShell>`
  - `<CommandPalette open: boolean onOpenChange(o: boolean) />`: on select it navigates to `/tool/$toolId`.
  - `<TopBar onOpenPalette(): void />`

- [ ] **Step 1: Failing tests** (render inside a memory router):
  - Pressing `Meta+k` opens the palette (`role="dialog"`).
  - Typing `shrink` shows `Compress PDF` first.
  - `Enter` navigates to `/tool/compress-pdf`.
  - Typing `zzzz` shows `No tool matches "zzzz".`
  - `Escape` closes the palette and returns focus to the trigger.
  - The top bar shows the text `LOCAL · 0 UPLOADS` and a theme toggle button with `aria-label="Switch to paper theme"` (dark) or `"Switch to dark theme"` (paper).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Top bar:** 48px, hairline bottom. The left holds the mark and wordmark (a `Link` to `/`). The centre holds the palette trigger styled as a `> find a tool, or drop a file` input with a `⌘K` mono hint. The right holds `● LOCAL · 0 UPLOADS` (dot `bg-signal`), the theme toggle and a settings `Link` to `/settings`.
  - **Palette:** `ui/command.tsx` (`cmdk`) with `shouldFilter={false}`, fed by `searchTools`. Each row shows `index`, name and format tag `{fromExt} → {toExt}` in mono.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit: `feat(shell): top bar and command palette`.

### Task 5: Home: hero, drop zone and tool index

**Files:**
- Create: `src/lib/pending-files.ts` (+ test), `src/components/home/drop-zone.tsx`, `src/components/home/tool-cell.tsx`, `src/components/home/tool-index.tsx`, `src/components/home/home.test.tsx`
- Modify: `src/routes/index.tsx`. Replace the page with the hero, `DropZone` and `ToolIndex`. Move the OCR, extract, export and settings sections out in Task 7, keeping them in `src/components/legacy-home-sections.tsx` until then.

**Interfaces:**
- Produces:
  - `setPendingFiles(files: File[]): void`, `takePendingFiles(): File[]` (returns the files and clears them), a module-level store.
  - `<DropZone onFiles(files: File[]): void />`
  - `<ToolIndex tools?: ConverterTool[] />` (defaults to all, grouped by `CATEGORY_ORDER`)
  - `<ToolCell tool: ConverterTool />`

- [ ] **Step 1: Failing tests.**
  - `takePendingFiles()` returns what was set, and `[]` on a second call.
  - Home renders `Every PDF tool.`, `None of the uploads.` and the footnote `¹ files processed on this machine. Network calls: 0.`
  - It renders section headers `01 ORGANIZE` through `09 OCR & DATA`.
  - Firing `drop` with `a.pdf` shows the heading `Tools for 1 file` with Merge PDF first.
  - Dropping `a.zip` shows `No tool takes .zip files.`
  - Clicking a routed tool calls `setPendingFiles` and navigates to `/tool/<id>`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Hero:** `font-display`, about 72px, `wdth` 75, weight 800, tracking −0.02em. The footnote is mono `text-ash`.
  - **Drop zone:** a hairline frame with four L-shaped crop marks in the corners (`--rule-strong`, 12px arms). On drag-over they snap inward 8px (300ms outExpo) and the marks turn signal. It includes a `Browse` button: `selectNativeFiles` under Tauri, a file `<input>` otherwise.
  - **Tool cell:** shows `index`, `name`, `description` (one line, clamped) and the format tag. The cell has a 1px `border-rule`, and on hover its top border turns `border-signal` and the index shifts 2px right.
  - **Layout:** a 4-column grid, 2 columns below 1100px.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5:** Commit: `feat(home): hero, smart drop zone, tool index`.

### Task 6: Tool workspace

**Files:**
- Create: `src/routes/tool.$toolId.tsx`, `src/components/workspace/file-stack.tsx`, `src/components/workspace/page-preview.tsx`, `src/components/workspace/tool-options.tsx`, `src/components/workspace/action-bar.tsx`, `src/components/workspace/result-row.tsx`, `src/components/workspace/options-state.ts` (+ test), `src/components/workspace/workspace.test.tsx`
- Modify: `src/routes/convert.$toolId.tsx` and `src/routes/convert.index.tsx` become redirects to `/tool/$toolId` and `/`.

**Interfaces:**
- Consumes: `executeConversion(toolId, file, options): Promise<ConversionResult>` and `renderUniversalPdfPage` from `converter-engine.ts`. `selectNativeFiles`, `showInNativeFolder` and `isTauri` from `tauri-bridge.ts`. `takePendingFiles`, `SparkProgress` and `StatusTag`.
- Produces:
  - `type ToolOptionsState` (the `useState` fields at `convert.$toolId.tsx:79-95`, as one object)
  - `defaultOptions(): ToolOptionsState`
  - `toConversionOptions(s: ToolOptionsState, extra: { additionalFiles: File[]; nativePath?: string }): ConversionOptions` (same mapping as `convert.$toolId.tsx:224-242`, with `parseNumbersList` moved here)
  - `runBlockReason(toolId: string, files: File[]): string | null`
  - `<PagePreview file: File maxEager?: number /* default 24 */ />`

- [ ] **Step 1: Failing tests.**
  - `toConversionOptions(defaultOptions(), {additionalFiles: []})` deep-equals the object the old page built from its initial state (`pageOrder: [1,2]`, `pagesToRemove: [1]`, `compressionLevel: "recommended"`, `watermarkText: "CONFIDENTIAL"`, …).
  - `runBlockReason("merge-pdf", [onePdf])` is `"Add at least 2 PDFs."`, and `null` with two.
  - `runBlockReason("compress-pdf", [])` is `"Add a file to start."`
  - The workspace given an `executeConversion` mock resolving `{success:false, error:"Encrypted PDF: password required"}` shows `ERR` and that message, with no `role="alertdialog"`.
  - `PagePreview` with a mocked 500-page document mounts ≤ 24 `canvas` elements.
  - A successful compress result shows a size delta like `−62%` (input 1000 bytes, `outputSizeBytes` 380).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.**
  - **Layout:** a grid of `260px | 1fr | 320px`. Left is the `FileStack`: reorder by drag (HTML5 DnD on list items), remove, add more, and size/pages in mono. The centre is `PagePreview` on `bg-paper` with hairline frames and IntersectionObserver lazy rendering. The right is `ToolOptions`, ported from the option controls in `convert.$toolId.tsx` and restyled with tokens.
  - **Action bar:** sticky at the bottom. `RUN ⏎` uses a signal fill, signal-ink text and 2px radius, and is disabled with the `runBlockReason` text beside it. ⏎ triggers it when not focused in a text field. While running it shows `SparkProgress` (indeterminate).
  - **Result row:** filename, size delta, then `Show in folder` (Tauri), `Open`/`Download` and `Run again`.
  - **Pending files:** call `takePendingFiles()` on mount.
  - **Panel tools:** tools with `panel` render the Task 7 panel instead of options and preview. Until Task 7 lands, render `StatusTag warn` "Moving here in this release."
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Browser check.** Run the dev server. In the browser pane:
  - Merge `test_samples/sample_digital.pdf` with itself and confirm it produces output.
  - Compress it and confirm a size delta shows.
  - Screenshot both, and look at them.
- [ ] **Step 6:** Commit: `feat(workspace): unified tool workspace at /tool/$toolId`.

### Task 7: OCR & Data tools and the settings page

**Files:**
- Create: `src/components/panels/ocr-panel.tsx`, `src/components/panels/extract-panel.tsx`, `src/components/panels/dataset-panel.tsx`, `src/routes/settings.tsx` (replaces the redirect), `src/components/panels/panels.test.tsx`
- Delete: `src/components/legacy-home-sections.tsx`
- Modify: `src/routes/ocr.tsx`, `src/routes/extract.tsx` and `src/routes/datasets.tsx` to redirect to `/tool/ocr`, `/tool/extract` and `/tool/dataset-export`

**Interfaces:**
- Consumes: `useOculus()` from `oculus-context.tsx` (unchanged), and the section JSX from `legacy-home-sections.tsx` (`#files`, `#ocr` → `OcrPanel`, `#extract` → `ExtractPanel`, `#export` → `DatasetPanel`, `#settings` → `/settings`).
- Produces: `<OcrPanel />`, `<ExtractPanel />`, `<DatasetPanel />`. `/settings` holds the theme (`useTheme`), the OCR languages and the default output folder, using the state the old settings section used.

- [ ] **Step 1: Failing tests.**
  - `/tool/ocr` renders `OcrPanel` (heading `OCR`) with a language select containing `Malayalam`.
  - `/settings` renders a theme radio with `Dark` and `Paper`, and choosing `Paper` sets `document.documentElement.dataset.theme === "paper"`.
  - `/ocr` redirects to `/tool/ocr`.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.** Move the JSX and handlers as is, then restyle: replace `timeline-*` classes with token utilities, and status chips with `StatusTag`. Malayalam text uses `font-ml`.
- [ ] **Step 4:** Run → PASS. Browser: OCR `test_samples/sample_mal.png` under `cargo run` (the Tauri app) and screenshot the result.
- [ ] **Step 5:** Commit: `feat(panels): ocr, extract, dataset tools and settings page`.

### Task 8: Restyle the remaining pages and delete the old code

**Files:**
- Modify: `src/routes/editor.tsx`, `src/routes/forms.tsx`, `src/routes/__root.tsx` (404 and error components), `src/components/work-progress-bar.tsx` (re-implement on `SparkProgress` or delete if unused)
- Delete: `src/routes/pricing.tsx`, `src/components/oculus-shell.tsx`, and `src/components/ui/{carousel,calendar,chart,menubar,navigation-menu,pagination,breadcrumb,context-menu}.tsx` with their deps if they're then unused (`embla-carousel-react`, `react-day-picker`, `recharts`).
- Modify: `src/styles.css`. Delete every unused selector (`timeline-*`, old shell classes). Target under 600 lines.

- [ ] **Step 1:** Write a test asserting `/pricing` renders the 404 component (`404`), and that no rendered page text matches `/neural|magic|AI-powered|seamless|unleash/i` (render `/`, `/tool/merge-pdf`, `/settings`, `/editor`, `/forms`). Run → FAIL.
- [ ] **Step 2:** Restyle and delete. Use `grep -rn "oculus-shell\|pricing\|timeline-" src` to confirm no references remain, and check `wc -l src/styles.css` < 600.
- [ ] **Step 3:** `bun run test && bun run lint && bunx tsc --noEmit && bun run build`. Expected: all exit 0.
- [ ] **Step 4:** Commit: `refactor: restyle editor/forms, remove pricing and dead UI code`.

### Task 9: Verification and integration

- [ ] **Step 1:** In the browser pane with the dev server, screenshot `/` in dark and paper at 1280×850 and 800×600, the open palette with `shrink`, a drop of `sample_digital.pdf`, and `/tool/compress-pdf` with a result. Look at each screenshot.

  Check:
  - there is no hue other than orange
  - there is no horizontal scroll at 800px
  - the crop marks and spark render

  Fix anything off in the owning component.
- [ ] **Step 2:** Run `grep -rniE "#(16a34a|d97706|dc2626)|green|emerald" src` and expect no matches.
- [ ] **Step 3:** Tauri: from the parent, `cd src-tauri && cargo run` with the dev server up. Confirm the window title `OCULUS — Araskova`, the new home, and that Repair PDF (native) runs on `sample_digital.pdf`. Confirm `cargo test` still passes, with 36 tests.
- [ ] **Step 4:** In the submodule, `git push -u origin feat/araskova-redesign` only if the user approves pushing. Otherwise leave it local. In the parent: `git add araskova-doccraft && git commit -m "chore: bump araskova-doccraft to redesign branch"`.
