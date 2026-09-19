# START PROMPT FOR THE CONSTRUCTION AGENT

You are now responsible for constructing OCULUS.

First, inspect the entire project directory.

CRITICAL: Look for DavidStack files/folders. DavidStack is a project-local stack of rules that will be placed into this project folder later and must be used to build accurate, high-quality products. If present, read it completely enough to identify all applicable construction rules before coding. If absent, continue with the supplied OCULUS rulebook and clearly record DavidStack as pending.

Then read:
- PRD.md
- ARCHITECTURE.md
- API_SPEC.md
- UI_SPEC.md
- IMPLEMENTATION_PLAN.md
- TEST_PLAN.md
- SECURITY.md
- AGENT_RULES.md
- ROADMAP.md
- DECISIONS.md

After inspection:
1. Produce a concise implementation checklist.
2. Identify the first smallest vertical slice.
3. Create the project skeleton.
4. Build the first working end-to-end OCR path.
5. Run tests/build/lint.
6. Fix failures before expanding scope.
7. Keep the architecture React → Tauri → Rust → C++ → AI/DSP/ML.
8. Do not substitute Python for production application logic without explicit justification.
9. Do not add cloud dependencies to core OCR.
10. Keep documentation synchronized with implementation.

Do not merely scaffold folders. The objective is a working, accurate product.
