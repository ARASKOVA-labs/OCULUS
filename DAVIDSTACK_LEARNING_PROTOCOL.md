# DavidStack Learning Protocol — Bug Knowledge & Gate Evolution

## Purpose
The DavidStack Learning Protocol establishes a disciplined engineering feedback loop between active application development and the canonical **DavidStack** rule layer. When an issue, edge-case failure, or architectural pitfall occurs during development or release auditing, this protocol ensures the learning is captured with empirical rigor, verified with regression tests, and translated into reusable engineering standards.

---

## 1. The Core Feedback Loop

```
OCULUS ISSUE (Observed Failure)
       ↓
ROOT CAUSE ANALYSIS (First Principles)
       ↓
FIX (Minimal, Clean, RAII/Safe Rust)
       ↓
REGRESSION TEST (Permanent Automated Check)
       ↓
GENERALIZED LESSON (Language / Architecture Level)
       ↓
DAVIDSTACK PROPOSAL (Banned Pattern, Gate Scanner, or Playbook)
       ↓
HUMAN REVIEW & APPROVAL
       ↓
CANONICAL DAVIDSTACK UPDATED
```

---

## 2. Bug Knowledge Capture Requirements

Every captured issue must be documented with the following mandatory fields:

1. **Symptom**: The exact observable external failure (e.g. panic message, memory leak, UI freeze, truncated output, silent data corruption).
2. **Reproduction**: Step-by-step instructions or test invocation that deterministically reproduces the issue.
3. **Root Cause**: The fundamental flaw at the code, compiler, OS, or ABI level. Speculation is disallowed; root cause must be backed by debugging output or stack traces.
4. **Fix**: The code modification applied to resolve the issue, strictly adhering to DavidStack rules (zero bare `.unwrap()`, zero raw `new`/`delete`, zero direct DOM manipulation).
5. **Verification**: Empirical proof that the fix resolved the issue without causing collateral regressions.
6. **Regression Test**: The automated test file and function name permanently committed to the repository.
7. **Generalizable Lesson**: The architectural or engineering principle that applies beyond this specific occurrence.

---

## 3. DavidStack Generalization Checklist

For every verified lesson, the engineer must answer the **Seven Generalization Questions**:

1. **Is this issue project-specific, or cross-cutting?**
2. **Could this happen in another project using the same technology stack?**
3. **Can an automated unit/integration test detect it?**
4. **Can an automated static gate scanner detect it?**
5. **Does DavidStack already have a rule or banned pattern that should have caught this?**
6. **If a rule exists, why was it insufficient?**
7. **Should a new banned pattern, gate scanner, or verification skill be proposed?**

---

## 4. Proposal vs. Mutation Rule

- **Canonical DavidStack is protected**: Agents and automated tools MUST NOT directly alter files in `David-Stack/core/` without human review.
- All additions must be drafted in `DAVIDSTACK_PROPOSALS.md` with rationale, regex patterns, affected languages, and severity (`block` or `warn`).
- Only after explicit human sign-off may the rule be merged into `David-Stack/core/gate/banned-patterns/`.
