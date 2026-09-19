# OCULUS Construction Agent Rules

You are the implementation agent for OCULUS.

## Rule 1 — DavidStack first
At the beginning of every construction session, inspect the project folder for DavidStack. It may be added later and may contain rules, standards, templates, checklists, scripts, or product-building constraints.

If DavidStack exists:
1. Read it before implementing.
2. Follow its applicable rules.
3. Do not overwrite or weaken its rules.
4. Integrate its requirements into the implementation plan.
5. If a DavidStack rule conflicts with an explicit OCULUS requirement, document the conflict and choose the higher-priority instruction.

If DavidStack is absent:
- Continue using this repository's documents.
- Do not fabricate DavidStack content.
- Leave a clear note that DavidStack is pending.

## Rule 2 — Architecture
React/TypeScript → Tauri → Rust → C++ → AI/DSP/ML.

Rust is the application layer.
C++ is the native performance engine.
Do not move general business logic into C++.
Do not make the frontend directly control C++.

## Rule 3 — Build in vertical slices
Prefer a small working end-to-end feature over large disconnected scaffolding.

## Rule 4 — No fake implementation
Do not create buttons that pretend to work.
Do not use mock success responses in production paths.
If a feature is incomplete, mark it explicitly.

## Rule 5 — Quality gates
Before declaring a feature complete:
- build
- lint
- format
- unit test
- integration test where applicable
- manually exercise the main user flow

## Rule 6 — Privacy
Core OCR is local-first.
Never introduce silent uploads, telemetry, analytics, or remote inference.

## Rule 7 — Documentation
Update architecture/API documentation when interfaces change.

## Rule 8 — Keep boundaries clean
Use typed IPC and typed FFI.
Keep modules small.
Avoid circular dependencies.

## Rule 9 — Performance
Do not optimize blindly.
Profile bottlenecks.
Avoid unnecessary memory copies.
Never block the UI.

## Rule 10 — Product quality
The finished product must feel like a real, polished product:
- useful empty states
- meaningful errors
- clear progress
- accessible interactions
- sensible defaults
- no dead controls
- no obvious placeholder UI

## Rule 11 — Stop conditions
Stop and request clarification if a requirement would:
- materially change the architecture
- weaken security/privacy
- contradict DavidStack
- require a missing proprietary dependency
- require an unsupported platform capability

Otherwise make reasonable engineering decisions and document them.
