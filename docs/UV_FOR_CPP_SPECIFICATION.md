# CUV (C-Ultra-Velocity): The "uv" for C/C++
### Architectural Specification & Project Blueprint for a Next-Generation C/C++ Toolchain Manager

---

## 1. Executive Summary & The Problem

In the modern systems programming landscape:
- **Rust** has `cargo` — unified, fast, standardized dependency resolution and compilation.
- **JavaScript/TypeScript** has `bun` — single-binary replacement for Node, npm, and Vite, starting in <10ms.
- **Python** has `uv` — single-binary Rust toolchain replacing pip, virtualenv, poetry, and pyenv, running 10–100x faster.

**C and C++ still live in the dark ages**:
- Developers wrestle with complex `CMakeLists.txt`, `vcpkg` submodules, slow Python-based `conan 2` recipes, or ad-hoc system packages (`apt`, `brew`, `pacman`).
- Build and configuration times are notoriously sluggish.
- Cross-platform dependency resolution requires hours of manual scripting.
- There is **no single binary, zero-config, ultra-fast toolchain and package manager for C++**.

**`cuv`** is designed to solve this: **A single, standalone Rust binary that acts as the universal package manager, build driver, and toolchain manager for C and C++**.

---

## 2. Core Pillars of `cuv`

```
┌─────────────────────────────────────────────────────────────┐
│                   CUV CLI (Single Rust Binary)              │
│        cuv init │ cuv add │ cuv run │ cuv build │ cuv test  │
└──────────────────────────────┬──────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       ▼                       ▼                       ▼
┌──────────────┐      ┌─────────────────┐     ┌──────────────────┐
│  Pubgrub C++ │      │ Content-Address │     │ Hermetic Clang/  │
│  Dependency  │      │  Binary & Header│     │ LLVM Toolchain   │
│   Resolver   │      │ Cache (Hardlink)│     │ Provisioner      │
└──────────────┘      └─────────────────┘     └──────────────────┘
       │                       │                       │
       └───────────────────────┼───────────────────────┘
                               ▼
        ┌─────────────────────────────────────────────┐
        │       ABI-Aware Composite Matrix Hasher     │
        │  SHA256(Source + Compiler + Std + Flags)    │
        └─────────────────────────────────────────────┘
```

### Pillar 1: Single Statically Linked Rust Binary
- Zero runtime dependencies: No Python, no Perl, no CMake, no Ninja required.
- Cold boot time: **< 5ms**.
- Cross-platform: macOS (Apple Silicon + Intel), Linux (x86_64, ARM64, musl), Windows (MSVC, MinGW).

### Pillar 2: Standardized Declarative Manifest (`cxx.toml`)
No more procedural CMake scripting for simple projects. Clean, declarative configuration inspired by `Cargo.toml` and `pyproject.toml`:

```toml
[project]
name = "oculus_engine"
version = "0.1.0"
standard = "c++20"
type = "static-lib" # or "executable", "shared-lib", "header-only"

[dependencies]
fmt = "^10.2.0"
nlohmann_json = "^3.11.3"
spdlog = "^1.13.0"
tesseract = ">=5.5.0"
leptonica = ">=1.87.0"

[dev-dependencies]
catch2 = "^3.5.0"

[target.'cfg(target_os = "macos")']
frameworks = ["CoreGraphics"]
flags = ["-fobjc-arc"]

[target.'cfg(target_os = "linux")']
links = ["pthread", "dl"]
```

---

## 3. Solving the Hardest Problem in C++: ABI Compatibility

The reason previous C++ package managers failed or remained slow is the **C++ ABI matrix**. Unlike Python wheels or npm JS bundles, a C++ binary compiled with GCC 11 on Ubuntu cannot safely link against a library compiled with Clang 18 on Alpine.

`cuv` solves this with **Canonical Composite Matrix Hashes**:

```rust
pub struct AbiFingerprint {
    pub package_name: String,
    pub version: String,
    pub source_git_commit: [u8; 20],
    pub compiler_id: CompilerId,       // e.g. Clang, GCC, MSVC, AppleClang
    pub compiler_version: SemVer,     // e.g. 18.1.3
    pub target_triple: String,        // e.g. aarch64-apple-darwin
    pub cxx_standard: CxxStandard,    // e.g. C++17, C++20, C++23
    pub stdlib: StdLib,               // libc++, libstdc++, msvcrt
    pub optimization: OptLevel,       // O0, O2, O3, Os, Oz
    pub compile_definitions: Vec<String>,
    pub position_independent: bool,   // -fPIC
}
```

- **Global Content-Addressable Cache (`~/.cuv/cache/`)**:
  - If a package matching the exact `AbiFingerprint` has already been compiled on this machine, `cuv` creates a filesystem **hardlink** into the project's build directory in **< 1ms**.
  - **Zero recompilation of unchanged libraries across any project on the machine.**

---

## 4. Zero-Friction Package Categories

`cuv` classifies dependencies into three fast lanes:

### Lane A: Header-Only Packages (Zero Compilation)
- Over 50% of popular modern C++ libraries (`nlohmann_json`, `glm`, `stb`, `eigen`, `catch2`, `magic_enum`, `expected`) are pure headers.
- `cuv add github:nlohmann/json` fetches the Git tag or release tarball, caches the `include/` tree globally, and hardlinks it into the project header search path in **under 200ms**.

### Lane B: Pre-Compiled Binary Artifacts
- For heavy libraries (`tesseract`, `leptonica`, `opencv`, `llvm`, `boost`):
- `cuv` queries a public, CDN-accelerated registry containing pre-compiled static and shared libraries indexed by target triple and compiler version.
- Download, unpack, and link without building from scratch.

### Lane C: Source-Built Ports (Automated Hermetic Build)
- Fallback for custom compile flags:
- `cuv` compiles using an embedded, high-concurrency compilation runner (Rust-native jobserver with `kqueue`/`epoll`).

---

## 5. CLI Command Suite (The UX of UV in C++)

| Command | Action | Speed |
| :--- | :--- | :--- |
| `cuv init [name]` | Scaffolds a modern C++20 project with `cxx.toml` and `.clang-format` | < 5ms |
| `cuv add <pkg>` | Resolves version, downloads/hardlinks, and adds to `cxx.toml` | < 100ms |
| `cuv run [args]` | Compiles modified sources incrementally and executes the binary | Sub-second |
| `cuv build --release`| Full optimized build with LTO and strip | Max hardware speed |
| `cuv test` | Discovers Catch2 / GTest / doctest test cases and runs in parallel | Parallelized |
| `cuv toolchain install <v>` | Downloads and manages hermetic LLVM/Clang toolchains | Isolated |
| `cuv export cmake` | Generates a standard `CMakeLists.txt` for legacy enterprise interop | Instant |

---

## 6. Implementation Roadmap to Build `cuv`

### Phase 1: MVP CLI & Manifest Parser (Week 1–2)
- Built in **Rust** using `clap`, `tokio`, `serde`, and `toml`.
- Parses `cxx.toml`.
- Native invocation of host `clang++` or `g++` with proper `-std=`, `-I`, `-L`, `-l` flags.
- Output `compile_commands.json` for instant `clangd` LSP integration in VS Code and Neovim.

### Phase 2: Content-Addressable Header Cache & Git Dependency Ingestion (Week 3–4)
- Resolves GitHub repository dependencies (`user/repo@v1.2.0`).
- Content-addressable storage in `~/.cuv/cache/src/<pkg>-<hash>/`.
- Instant hardlink/symlink mounting into `.cuv/include/`.

### Phase 3: Rust-Native Parallel Compiler Driver (Week 5–6)
- Replaces Ninja with an embedded DAG task scheduler written in Rust.
- Dependency tracking via compiler `-MMD` generation.
- Real-time terminal progress indicators (rich spinner + dopamine progress bars).

### Phase 4: Precompiled Binary Registry & Package Distribution (Week 7–8)
- GitHub Actions pipeline to pre-compile popular C++ libraries across:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
- S3/R2/GitHub Releases binary distribution with SHA256 verification.

---

## 7. Current Project Recommendations for OCULUS

While developing `cuv` as a sister open-source project:
1. **Frontend / Web / Node**: Use **`bun`** exclusively (`bun run dev`, `bun run build`, `bun install`).
2. **Python Tooling**: Use **`uv`** exclusively (`uv run python ...`).
3. **C++ Native Engine (`native/`)**:
   - For local builds: Use host Clang with `src-tauri/build.rs` via `cc::Build`.
   - The fastest existing tool: **`xmake`** (can be tested as an alternative to CMake).
   - Long term: Transition `native/` to **`cuv`** as its premiere reference showcase!
