# 12 — Implementation Plan: digital-product-studio

**Produced by:** SLOT-3 planner
**Date:** 2026-10-12
**Kanban task:** t_c420e098 (board: digital-product-studio)
**Status:** Complete. Slot-3.5 (orchestrator-pivot, t_377df471) fans this out into board rows.

---

## 1. What this plan is

The DAG-schema (fan-out spec) + per-chunk bodies + testing strategy for the 12-stage
canonical pipeline, stages 4→12, for the digital-product-studio project:

**pipeline-core** (Rust/axum REST + orchestrator) · **pipeline-workers** (job execution,
command-spec jail) · **admin-ui** (React 19 console) — deployed via Openship compose +
Pingap TLS, against shared-minio bucket `digital-product-studio`, marketplace adapters
Etsy / Lemon Squeezy / Printify / Gelato.

Upstream inputs (consumed, not restated):
- docs/09-requirements.md — R1–R15 (slot-1 researcher)
- docs/10-security-research.md — S1–S15, H1–H15, threat model, dependency audit (slot-1.5)
- docs/11-architecture.md — components, data model (12 tables), 23-route API surface,
  PlatformAdapter trait, Postgres SKIP LOCKED worker queue, curation flow, deployment
  view, ADR-001..006, version floors (slot-2 architect)

Deliverable files (all under `/config/.hermes/workspace/projects/digital-product-studio/docs/`):
- **12-implementation-plan.md** — this file (narrative plan + DAG summary)
- **12-dag-schema.md** — the ONLY thing slot-3.5 reads: 28 rows, titles, assignees,
  parent wiring map, verified skills, monitoring partition
- **12-chunk-bodies.md** — per-row skeleton: goal / inputs / outputs / skills / acceptance
- **12-testing-strategy.md** — per-module test matrix, security suites, e2e contract

MemPalace: wing=digital-product-studio, room=planner, drawers: dag-schema, chunk-bodies,
testing-strategy, planner-overview (+ KG facts + diary).

## 2. DAG summary (28 rows)

```
slot-3.5 (t_377df471)
└─ stage 4 (design-ux, 4 parallel):  4.1 curation+runs · 4.2 catalog+listings
                                      4.3 analytics+niche-board · 4.4 settings+auth shell
   ├─ stage 5 (frontend-engineer, 4): 5.1 shell+routing+API client · 5.2 query hooks
   │                                   5.3 curation+runs components · 5.4 catalog/listings/analytics components
   └─ stage 6 (backend-engineer, 4):  6.1 service shape · 6.2 queue+worker+jail
                                      6.3 adapters · 6.4 admin API + S3 layer   (5 ∥ 6)
      └─ stage 7 (coder, 10): 7.1 db+migrations · 7.2 queue/worker · 7.3 jail+S3
         7.4 etsy · 7.5 lemonsqueezy · 7.6 printify+gelato · 7.7 admin API core
         7.8 admin API rest · 7.9 admin-ui shell+pages · 7.10 admin-ui components
         (parents: 7.1–7.8 ← 6.x; 7.9–7.10 ← [5.4, 6.4])
         └─ 8 reviewer (all 7.x done) → 8.5 security (S1–S15) → 9 devops
            (Dockerfiles, compose, GH Actions, Openship, Pingap/cert, MinIO bucket)
            → 10 tester (execute all suites in Docker) → 11 qa-engineer
            (e2e smoke: run → generate → curate → assemble → publish-dry-run)
            → 12 documenter (README + ops runbook)
```

Sequencing is explicit in 12-dag-schema.md §1 (no stage can race ahead):
4.x parallel after 3.5 → 5.x and 6.x after all-4.x (parallel tracks) → 7.x after its
5/6 predecessors → 8 after ALL 7.x → 8.5 → 9 → 10 → 11 → 12, strictly linear.

## 3. Key planning decisions (planner-owned)

1. **Chunk counts = defaults kept (4/4/4/10/6 singles = 28 rows).** No chunk exceeds
   scope limits; the largest coder chunks (7.7, 7.8) stay ≤ ~8KB of planned code by
   reusing 6.x handler specs and 7.1 query modules.
2. **Stage-8.5 assignee = `security` profile** (no `security-reviewer` profile exists on
   this install — verified `/config/.hermes/profiles/`). Titles say "security"; slot-3.5
   must use assignee `security`.
3. **7.9/7.10 multi-parent [5.4, 6.4]:** admin-ui code needs BOTH the frontend-engineer
   component specs and the backend DTO shapes; single-parent wiring would let 7.9 start
   before 6.4 exists and produce a mismatched API client.
4. **Reviewers/testers do NOT edit code.** Stage 8/8.5/10/11 are read-only gates that
   file evidence drawers and block-with-reason; fixes route back through the kanban
   dispatcher as rework on the offending 7.x row.
5. **Mock-platform testing everywhere:** adapters are contract-tested against in-Docker
   mock servers; stage 11 asserts zero external egress during publish-dry-run. No real
   marketplace credentials are exercised until post-MVP manual operator onboarding.
6. **Docker-only enforcement is repeated in every row body** via the 4 HARD CLAUSES
   (slot-3.5 template) — code authoring on host is allowed; every build/test/install
   runs in containers; GH Actions is the only real build pipeline (images pulled by
   Openship, never built on host).
7. **Skills pinned to the verified-resolving set** (12-dag-schema.md §4, probed on disk
   2026-10-12). Universal fallback available on all profiles if a row needs one:
   `document-driven-implementation` + `development-rules` + `development/universal-development-standards`.

## 4. Risk notes carried into stages 8–11

- **Cargo.lock floors must be re-verified at first build** (S4–S6) — stage 10 asserts
  `cargo tree` inside the build container (docs/11 "known UNVERIFIED items").
- **Gelato rate cap undocumented** (slot-1.5 §7) — 7.6 implements conservative 4 rps +
  jittered backoff; do not "fix" this upward without research.
- **MinIO bucket policy + Pingap myCA choice** are deploy-time verifications (stage 9
  operator steps with `mc policy get` + openssl S12 probe).
- **cf-image-gen invocation contract** (docs/09 §3) is the worker's single generation
  backend; stage 11 uses its placeholder/test mode — never a paid API in CI.

## 5. Handoff

slot-3.5 reads 12-dag-schema.md (drawers: dag-schema) and creates the 28 rows with the
parent wiring map, pastes chunk bodies from 12-chunk-bodies.md, embeds the 4 HARD
CLAUSES verbatim in every row, then creates ceil(28/5)=6 monitoring-board tasks and
reports created ids grouped by stage.

---

*End of implementation plan. Companion files: 12-dag-schema.md · 12-chunk-bodies.md · 12-testing-strategy.md*
