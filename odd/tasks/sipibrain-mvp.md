# SIPI-brain MVP — T001–T028
Objective: implement only the authorized Rust/Vue MVP. Branch: codex/sipibrain-mvp (local repository setup pending). Delivery: single-pr planning boundary, local work only; no push or PR authorized. Runners: Cargo tests/check/clippy; Vue typecheck/Vitest/build; local browser/API smoke.

## Specs

- S1: "$speckit-implement implementa el MVP de T001 a T028 siguiendo el plan de Rust y Vue 3 + TypeScript." Authorized scope is exactly the existing T001–T028 in specs/001-sipibrain-collective-memory/tasks.md; read the full spec, plan, model and contracts for their acceptance criteria. T029 onward remains unimplemented.
- S2: "Backend: Rust. Frontend: Vue 3 + TypeScript." Preserve the current constitution and all source/assignment/proposal/review invariants; only synthetic data, no service connections or secret operations.
- S3: Checkpoint proof must match the current tasks, including C1 startup/static Vue and C2 project-query contract preparation in T003. Full project-search story US3 is not part of this MVP.

## Tasks

- M1 | S1–S3 | delegated setup checkpoints: backend and frontend are independent contract-bound components with disjoint surfaces, each continuing with its tests as a multi-minute behavior unit | pending | commit pending
- M2 | S1–S3 | parent contract/DTO seam, then delegated API/session/assignment and Vue-session components, parent integration | pending | commit pending
- M3 | S1–S3 | delegated knowledge/proposal API and contract-bound Vue editing/review/conflict interface, then independent security/concurrency verification and local smoke | pending | commit pending

## Log

- L1: $speckit-implement implementa el MVP de T001 a T028 siguiendo el plan de Rust y Vue 3 + TypeScript.
- 2026-11-16: requirements checklist 16/16; no extension hooks. No application exists. Rust 1.98.0, Node 24.22.1/npm 11.19.1 work with reviewed external execution; restricted profile cannot resolve home/tool settings. No global Node version changed.
- 2026-11-16: Git resolves an enclosing HOME repository; do not stage or commit there. Create a local isolated repository/feature branch before any work-unit commits. Preserve all existing docs and instructions. No remote Git operations authorized.
- Forecast: cohesive MVP likely exceeds 400 authored lines; preserve reviewable behavior units and tests, do not omit scope for a line target. Delivery is one locally requested MVP, single-pr planning only; any actual PR/push remains a separate user decision. Engram mirror pending because no canonical project resolved.
