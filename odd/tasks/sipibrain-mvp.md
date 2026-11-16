# SIPI-brain MVP — T001–T028
Objective: implement only the authorized Rust/Vue MVP. Branch: codex/sipibrain-mvp. Delivery: single-pr planning boundary, local work only; no push or PR authorized. Runners: Cargo tests/check/clippy; Vue typecheck/Vitest/build; local browser/API smoke.

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
- 2026-11-16: isolated repository baseline 5d75252; Engram now resolves canonical project llmwiki. Review mode on globally. T003 contract now specifies exact local maintenance/review/source/link/event payloads and optional general project query; backend and frontend setup are disjoint waited workers, no later-story work.
- 2026-11-16: T001/T002 completed with Cargo check/fmt and Vue typecheck/Vitest(1)/build. Parent repeated frontend typecheck successfully. Work now limited to T003 DTOs/harness and T004/T005 security RED; no functional completion claimed.
- 2026-11-16: T003-T005 observed contract/DTO/harness and assertion-based RED. T006-T009 now authorized to backend worker. Parent observed query DTO seam mismatch text/q and fixed contract choice q before implementation. Frontend functional implementation waits for foundation API.
- 2026-11-16: T006-T009 GREEN: check/clippy/full Cargo11; parent repeated security7 PASS. Backend phase done, T010/T011 to frontend; T012 actual compiled Vue smoke pending. Environment: SIPIBRAIN_ORIGIN exact application origin, optional SIPIBRAIN_DEV_ORIGIN exact loopback5173 only. HTTP synthetic-only.
- 2026-11-16: T010/T011 GREEN with six assertion RED before implementation, Vitest7/typecheck/build pass. T012 actual compiled Vue API smoke delegated per-action to backend worker, then RED T013-015 only. Native assess attempted with named security risk2: high/unassessable because implementation files untracked; no authority frozen or consent inferred; assessment/review will run on finalized local unit.
- 2026-11-16: T012 verified actual compiled Vue root/JS200 and unknownAPI404JSON; full Rust11/Vue7/typecheck/build pass. T013-T015 observed7 assertion RED on missing source/record behavior; frontend T016 RED now authorized. Core implementation waits for all tests.
- 2026-11-16: T016 observed5 assertion RED/typecheck PASS. Backend authorized T017-T023 only, sequential core implementation with full immutable source/revision/reference/review/proposal/transaction invariants. T024+ waits for working API. Scope still no US2+, secrets or external services.
