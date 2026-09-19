# DavidStack Rule & Gate Proposals

**Source**: OCULUS Release Audit & Chaos Testing  
**Status**: DRAFT FOR HUMAN REVIEW (Canonical DavidStack in `David-Stack/` remains unmodified)  
**Protocol Compliance**: [DAVIDSTACK_LEARNING_PROTOCOL.md](file:///Users/anvinpshibu/Documents/Projects-Araskova/oculus_project_docs/DAVIDSTACK_LEARNING_PROTOCOL.md)

---

## Proposal 1: Banned Pattern — `no-raw-c-abi-boundary` (C++)

### Background & Seven-Question Evaluation
1. **Is this OCULUS-specific?** No. Any project using a C++ native engine called via C-ABI from Rust, Go, Python, or Swift suffers from this.
2. **Could this happen in another project?** Yes. Unhandled C++ exceptions escaping across `extern "C"` boundaries cause undefined behavior and immediate process abort via `std::terminate`.
3. **Can a test detect it?** Yes, by fuzzing the C-ABI with invalid inputs.
4. **Can an automated gate detect it?** Yes, by checking that exported `extern "C"` functions contain an enclosing `try/catch` block.
5. **Does DavidStack already have a rule?** `David-Stack/core/gate/banned-patterns/cpp.yaml` currently only bans `no-raw-new-delete` and `no-todo-merge`. It lacks an FFI exception safety rule.
6. **Why is the existing rule insufficient?** Memory safety rules do not prevent uncaught exceptions from terminating the host process.
7. **Should a new pattern be proposed?** Yes.

### Proposed YAML Entry for `core/gate/banned-patterns/cpp.yaml`
```yaml
  - id: no-raw-c-abi-boundary
    pattern: 'extern\s+"C"\s*\{[^}]*?[^t][^r][^y]\s*\{'
    reason: "Exported C-ABI functions must wrap all implementation code in a try/catch barrier to prevent process termination across FFI."
    severity: block
```

---

## Proposal 2: Banned Pattern — `no-hardcoded-dev-paths` (Rust / C++)

### Background & Seven-Question Evaluation
1. **Is this OCULUS-specific?** No. Developers frequently hardcode `/opt/homebrew` or `/usr/local` paths during local testing, causing builds to fail on end-user clean machines or CI containers.
2. **Could this happen in another project?** Yes, very common in native desktop and CLI projects.
3. **Can a test detect it?** Only if run on a clean machine without those paths.
4. **Can an automated gate detect it?** Yes, a regex pattern scanner can flag hardcoded package manager prefixes in non-test source files.
5. **Does DavidStack already have a rule?** No.
6. **Why is the existing rule insufficient?** No rule currently audits path portability.
7. **Should a new pattern be proposed?** Yes.

### Proposed YAML Entry for `core/gate/banned-patterns/rust.yaml`
```yaml
  - id: no-hardcoded-dev-paths
    pattern: '("(/opt/homebrew|/usr/local/Cellar|/Users/|/home/)[^"]*")'
    reason: "Hardcoded developer paths break clean-machine portability. Use dynamic discovery cascades, bundle-relative paths, or environment variables."
    whitelist: 'tests|benches'
    severity: block
```

---

## Proposal 3: Automated Verification Skill — `verify_offline_guarantee`

### Background & Seven-Question Evaluation
1. **Is this OCULUS-specific?** No. Many enterprise, privacy-centric, or healthcare applications mandate air-gapped, zero-telemetry execution.
2. **Could this happen in another project?** Yes. Dependencies can inadvertently introduce telemetry (e.g. Sentry, PostHog, telemetry beacons) without the developer's explicit intent.
3. **Can a test detect it?** Yes, by inspecting symbols in compiled production binaries.
4. **Can an automated gate detect it?** Yes, as a verification skill in `core/verification/`.
5. **Does DavidStack already have a rule?** `AGENT_RULES.md` has Rule 6 (Privacy: "Never introduce silent uploads, telemetry, analytics"), but lacks an automated verification script to enforce it.
6. **Why is the existing rule insufficient?** Written guidelines require automated static/symbol scanners to guarantee compliance in CI.
7. **Should a new skill be proposed?** Yes.

### Proposed Verification Skill Definition
- **Location**: `David-Stack/core/verification/skills/verify_offline_guarantee.py`
- **Logic**:
  ```python
  import subprocess, sys

  def verify_offline(binary_path):
      banned_symbols = [
          "analytics", "telemetry", "mixpanel", "posthog",
          "google_analytics", "segment_io", "datadog"
      ]
      result = subprocess.run(["strings", binary_path], capture_output=True, text=True)
      matches = [s for s in banned_symbols if s in result.stdout.lower()]
      if matches:
          return {"pass": False, "detail": f"Telemetry symbols detected: {matches}"}
      return {"pass": True, "detail": "Zero telemetry symbols verified"}
  ```

---

## Proposal 4: Playbook Enhancement — `desktop-packaging-checklist`

### Background
Add a formal release checklist to `David-Stack/core/david-mode/playbooks/release.md`:
1. Verify dynamic library paths using `otool -L` (macOS) or `ldd` (Linux) — ensure all libraries resolve to `@rpath` or system runtime.
2. Verify all application assets (`tessdata`, fonts, web assets) exist relative to the executable bundle.
3. Run test suites with simulated network disconnection to prove offline resilience.
4. Verify installer cleanup (uninstall removes application binaries without leaving lingering daemons).

---

## Proposal 5: Automated Verification Skill — `verify_clean_machine_bundle`

### Background & Seven-Question Evaluation
1. **Is this OCULUS-specific?** No. Any native desktop app (Tauri, Electron with native add-ons, Qt, C++/Rust) distributed on macOS or Linux encounters dynamic library linkage failures when built on developer machines.
2. **Could this happen in another project?** Yes. Very common failure mode in desktop software where binaries link to developer paths (`/opt/homebrew`, `/usr/local/Cellar`).
3. **Can a test detect it?** Yes, by running an automated audit of `otool -L` / `ldd` outputs and launching the binary inside a sanitized environment (`PATH=/usr/bin:/bin`, 0 developer environment variables).
4. **Can an automated gate detect it?** Yes.
5. **Does DavidStack already have a rule?** DavidStack has banned patterns for source files, but no release gate for distributable bundles.
6. **Why is the existing rule insufficient?** Source file scanners cannot detect link-time dependencies injected by build scripts (`build.rs`) or system linkers.
7. **Should a new skill be proposed?** Yes.

### Proposed Verification Skill Definition
- **Location**: `David-Stack/core/verification/skills/verify_clean_machine_bundle.py`
- **Logic**:
  Inspects the distributable `.app` or binary bundle:
  1. Audits all Mach-O binaries in `Contents/MacOS/` and `Contents/Frameworks/` using `otool -L`.
  2. Asserts zero occurrences of non-system paths (e.g. `/opt/homebrew/`, `/usr/local/`, `/nix/store/`).
  3. Launches the binary with a 2-second timeout under an empty environment (`env -i PATH=/usr/bin:/bin HOME=/tmp`) to ensure dynamic linking succeeds without crashes.

---

*Submitted for Human Review by Antigravity Engineering.*
