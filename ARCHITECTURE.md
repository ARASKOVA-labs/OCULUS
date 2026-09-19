# OCULUS Architecture

## Canonical flow

React / TypeScript
        ↓
Tauri
        ↓
Rust Application Layer
        ↓
C++ Native Engine
        ↓
OpenCV / OCR / PDF / optional ONNX Runtime

## Responsibilities

### React + TypeScript
Presentation only:
- screens
- components
- state presentation
- progress
- result visualization
- user interaction

Never perform privileged filesystem operations or unrestricted native execution.

### Tauri
Secure IPC and desktop integration.
Expose only typed, explicitly permitted commands.

### Rust
Application brain:
- file ingestion
- validation
- job queue
- worker management
- configuration
- persistence
- export orchestration
- IPC
- native engine lifecycle
- error mapping

### C++
Performance engine:
- OpenCV preprocessing
- OCR backend integration
- PDF/image operations where native libraries are required
- performance-critical algorithms

### AI/ML
Optional local inference:
- ONNX Runtime
- other native model runtimes
- future document layout or handwriting models

Models must be replaceable.

## FFI rules
- Keep Rust↔C++ boundary narrow.
- Prefer a stable C ABI or the `cxx` crate.
- Clearly define ownership and lifetime.
- Never pass ambiguous ownership across the boundary.
- Map C++ errors into structured Rust errors.
- Test the FFI boundary independently.

## Repository proposal

oculus/
├── README.md
├── PRD.md
├── ARCHITECTURE.md
├── API_SPEC.md
├── UI_SPEC.md
├── IMPLEMENTATION_PLAN.md
├── TEST_PLAN.md
├── SECURITY.md
├── AGENT_RULES.md
├── davidstack/                 # added later by project owner
├── frontend/
├── src-tauri/
├── native/
│   ├── cpp/
│   └── include/
├── models/
├── tests/
└── docs/

The actual repository may adapt this structure if DavidStack prescribes a different structure.
