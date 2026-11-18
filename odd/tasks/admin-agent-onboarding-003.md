# Spec 003 — Admin agent onboarding
Branch: codex/003-admin-agent-onboarding. Delivery: local diff only (no commits or push).
Runners: Cargo 1.98.0 offline/locked; Node 24 with Vitest, vue-tsc and Vite.

## Specs
S1. “Empieza por el alcance local en `codex/003-admin-agent-onboarding`” — implement and verify the local scenarios in [spec 003](../../specs/003-admin-agent-onboarding/spec.md) and its API contract; retain existing wiki and assignment behavior.
S2. “deja SC-004 y T025 pendientes de evidencia” — do not contact real services or claim delivery acceptance.
S3. “No hagas commit ni push todavía.” — keep all work local and uncommitted.

## Tasks
- T1 (S1): T001–T002 baseline and reuse; inline; verified.
- T2 (S1): T003–T011 registration and existing wiki workflow; inline; RED/GREEN observed and verified.
- T3 (S1): T012–T019 local connection workflow; read-only mapping delegated for catalog and loopback seams, implemented inline; RED/GREEN observed and verified.
- T4 (S1–S3): T023–T024 local verification passed; T020–T022 and T025 outside current execution.

## Log
L1. “$speckit-implement para la spec 003. Empieza por el alcance local en `codex/003-admin-agent-onboarding`; deja SC-004 y T025 pendientes de evidencia. No hagas commit ni push todavía.”
L2. Requirements checklist 16/16; existing Rust wiki administration 4/4 and Vue creation 1/1 before edits. Rust local stable reports 1.98.0; Node 24.22.1. Reuse temporary SQLite/session and simulated fetch.
L3. Local verification: 35 Rust default tests, 8 focused Rust fixture tests, 18 Vitest tests and frontend build/typecheck passed. Chromium validated the empty install, registration, wiki creation, restart and connection outcomes; four negative fixture activation modes and zero contacts after revocation passed. No real service access. Independent verification and targeted recheck passed; no commits or push.
L4. Independent probe found a revoke/restore edge case (identical source approval). The backend now tracks continuous per-entry approval epochs. Regression test, independent targeted recheck and parent replay returned not_testable after the correction.
L5. Native bounded correction: reject URL dot-segment IDs without persisting them, ignore stale wiki-list responses, and lock the local revoke read-modify-publish operation. Regression RED observed for all three; 20 Vitest tests, frontend typecheck/build, targeted Rust tests and concurrent revoke proof passed.
L6. Separate REG-001 fix: the fixture uses an OS-owned loopback listener during each revoke, not a persistent lock file. The regression killed a real revoke process before publication; the retry succeeded without cleanup. RED reproduced on the file lock, and all 20 Vitest tests plus frontend typecheck/build passed with the replacement.
