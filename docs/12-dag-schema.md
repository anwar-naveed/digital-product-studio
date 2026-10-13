# 12 — DAG-Schema: digital-product-studio (stages 4→12)

**Produced by:** SLOT-3 planner (kanban t_c420e098)
**Date:** 2026-10-12
**Status:** Complete — the ONLY input slot-3.5 (orchestrator-pivot, kanban t_377df471) reads to create stage-4+ board rows.

Consumes: docs/09-requirements.md (R1–R15), docs/10-security-research.md (S1–S15, H1–H15, M/T), docs/11-architecture.md (§1–§10, ADRs, version pins). Project root: `/config/.hermes/workspace/projects/digital-product-studio/`.

---

## 1. Sequencing constraints (verbatim contract for slot-3.5)

1. **Stage-4 chunks (4.1–4.4) run parallel, all parented to slot-3.5's task id** (t_377df471). No stage beyond 4 may parent to slot-3.5.
2. **Stage-5 chunks (5.1–5.4) start only after ALL stage-4 rows are done.** All 5.x share ONE parent = stage-4's LAST-created row id (4.4).
3. **Stage-6 chunks (6.1–6.4) start only after ALL stage-4 rows are done** (parallel with 5.x). All 6.x share ONE parent = stage-4's LAST-created row id (4.4).
4. **Stage-7 chunks start after their 5/6 predecessors:**
   - 7.1–7.3 (backend path) parent = stage-6's last row (6.4); additionally 7.2/7.3 require 6.2.
   - 7.4–7.6 (adapters) parent = stage-6's last row (6.4); additionally each requires 6.3.
   - 7.7–7.8 (admin API) parent = stage-6's last row (6.4); additionally each requires 6.4 + 6.1.
   - 7.9–7.10 (admin-UI code) parent = stage-5's last row (5.4); additionally 7.9 requires 5.1+5.2, 7.10 requires 5.3+5.4. NOTE: 7.9/7.10 also consume stage-6 outputs (API client shapes from 6.4) — they must not start before BOTH 5.x-all-done AND 6.4-done. slot-3.5 implements this by giving 7.9/7.10 parents = [5.4, 6.4] (multi-parent fan-in).
   - 7.1 additionally requires 5.x only via catalog-page contracts; no extra parent needed.
5. **Stage-8 (reviewer) after ALL 7.1–7.10 done.** Parent = last 7.x row (7.10) with multi-parent fan-in listing all ten 7.x ids where the CLI supports it; minimum: parent = 7.10.
6. **Stage-8.5 (security) after 8.** Parent = stage-8 row id.
7. **Stage-9 (devops) after 8.5.** Parent = stage-8.5 row id.
8. **Stage-10 (tester) after 9.** Parent = stage-9 row id.
9. **Stage-11 (qa-engineer) after 10.** Parent = stage-10 row id.
10. **Stage-12 (documenter) after 11.** Parent = stage-11 row id.

Total: 28 rows (4+4+4+10+1+1+1+1+1+1). Default chunk counts kept — no chunk exceeds scope limits (each coder chunk ≤ ~8KB planned code).

---

## 2. DAG rows (exact task titles for slot-3.5)

| # | Stage | Exact task title | Assignee |
|---|---|---|---|
| 1 | 4.1 | design-ux 4.1: Curation grid + production runs (core console UX) | design-ux |
| 2 | 4.2 | design-ux 4.2: Catalog + listings pages | design-ux |
| 3 | 4.3 | design-ux 4.3: Analytics + niche board | design-ux |
| 4 | 4.4 | design-ux 4.4: Settings + auth shell | design-ux |
| 5 | 5.1 | frontend-engineer 5.1: App shell + routing + API client | frontend-engineer |
| 6 | 5.2 | frontend-engineer 5.2: State/data layer (TanStack Query hooks per domain) | frontend-engineer |
| 7 | 5.3 | frontend-engineer 5.3: Curation + runs UI components | frontend-engineer |
| 8 | 5.4 | frontend-engineer 5.4: Catalog + listings + analytics components | frontend-engineer |
| 9 | 6.1 | backend-engineer 6.1: pipeline-core service shape (axum app, config, DB pool, migrations framework) | backend-engineer |
| 10 | 6.2 | backend-engineer 6.2: Job queue + worker runner + command-spec jail | backend-engineer |
| 11 | 6.3 | backend-engineer 6.3: Platform adapters (Etsy + LS + Printify/Gelato traits + webhook ingestion) | backend-engineer |
| 12 | 6.4 | backend-engineer 6.4: Admin REST API surface + S3 client layer | backend-engineer |
| 13 | 7.1 | coder 7.1: db schema + migrations + entities | coder |
| 14 | 7.2 | coder 7.2: job queue + worker loop | coder |
| 15 | 7.3 | coder 7.3: command-spec validators + S3 staging | coder |
| 16 | 7.4 | coder 7.4: etsy adapter | coder |
| 17 | 7.5 | coder 7.5: lemonsqueezy adapter | coder |
| 18 | 7.6 | coder 7.6: printify + gelato adapters | coder |
| 19 | 7.7 | coder 7.7: admin API niches + runs + curation | coder |
| 20 | 7.8 | coder 7.8: admin API catalog + listings + analytics | coder |
| 21 | 7.9 | coder 7.9: admin-ui shell + pages | coder |
| 22 | 7.10 | coder 7.10: admin-ui components | coder |
| 23 | 8 | reviewer 8: verify implementation vs architect decisions (docs/11) | reviewer |
| 24 | 8.5 | security 8.5: verify vs S1–S15 + harden profile | security |
| 25 | 9 | devops 9: Dockerfiles + docker-compose + GH Actions + Openship prep + Pingap/cert/bucket | devops |
| 26 | 10 | tester 10: test plan + run inside Docker | tester |
| 27 | 11 | qa-engineer 11: e2e smoke (run → generate → curate → assemble → publish-dry-run) | qa-engineer |
| 28 | 12 | documenter 12: README + ops runbook | documenter |

**Chunk-persist discipline (every row):** worker persists deliverables to MemPalace `wing=digital-product-studio`, `room=<stage-name>` (rooms: design-ux, frontend-engineer, backend-engineer, coder, reviewer, security, devops, tester, qa-engineer, documenter). Coder chunks ALSO write real files under the project root. Every row embeds the 4 HARD CLAUSES (environment hygiene) verbatim, the worker-mandatory-instructions block, and heartbeat ≥10m for long jobs.

---

## 3. Parent wiring map (slot-3.5 executes this exactly)

| Row(s) | Parent(s) |
|---|---|
| 4.1–4.4 | t_377df471 (slot-3.5) |
| 5.1–5.4 | 4.4's task id |
| 6.1–6.4 | 4.4's task id |
| 7.1 | 6.4's task id |
| 7.2 | [6.4, 6.2] |
| 7.3 | [6.4, 6.2] |
| 7.4 | [6.4, 6.3] |
| 7.5 | [6.4, 6.3] |
| 7.6 | [6.4, 6.3] |
| 7.7 | [6.4, 6.1] |
| 7.8 | [6.4, 6.1] |
| 7.9 | [5.4, 6.4] |
| 7.10 | [5.4, 6.4] |
| 8 | 7.10's task id (fan-in note: verify all ten 7.x done before starting) |
| 8.5 | 8's task id |
| 9 | 8.5's task id |
| 10 | 9's task id |
| 11 | 10's task id |
| 12 | 11's task id |

Board: `digital-product-studio`. All rows default `todo` (parent-gated by dispatcher). Never create a ROOT row.

---

## 4. Skills verified on target profiles (probe output, 2026-10-12)

Every skill below was confirmed to exist under `/config/.hermes/profiles/<assignee>/skills/`:

- **design-ux**: `design/ui-ux-pro-max`, `development/react-best-practices`, `development/frontend-development-patterns`, `development-rules`, `document-driven-implementation`
- **frontend-engineer**: `development/react-best-practices`, `development/frontend-development-patterns`, `development/universal-development-standards`, `development/testing-and-tdd`, `frontend/shadcn-ui`, `development-rules`, `document-driven-implementation`
- **backend-engineer**: `development/backend-development-patterns`, `development/universal-development-standards`, `development/testing-and-tdd`, `development-rules`, `document-driven-implementation`
- **coder**: `development/universal-development-standards`, `development/backend-development-patterns`, `development/react-best-practices`, `development/frontend-development-patterns`, `frontend/shadcn-ui`, `development/testing-and-tdd`, `document-driven-implementation`, `development-rules`
- **reviewer**: `software-development/code-analysis`, `software-development/requesting-code-review`, `development/universal-development-standards`, `document-driven-implementation`
- **security**: `software-development/code-analysis`, `software-development/systematic-debugging`, `development/universal-development-standards`, `document-driven-implementation`, `development-rules`
- **devops**: `devops/docker-expert`, `devops/openship-deployment`, `devops/docker-nginx-shared-edge`, `devops/monitoring-stack-integration`, `github/github-repo-management`, `devops/development-rules`
- **tester**: `software-development/test-driven-development`, `development/testing-and-tdd`, `software-development/code-analysis`, `development/universal-development-standards`, `document-driven-implementation`
- **qa-engineer**: `software-development/dogfood`, `software-development/test-driven-development`, `software-development/systematic-debugging`, `development/testing-and-tdd`, `software-development/code-analysis`
- **documenter**: `document-driven-implementation`, `development-rules`, `github/github-repo-management`

Do NOT cite any skill outside this verified list on the row's `skills` flag (cycle-29 lesson: unresolvable skill names crash workers at spawn).

**NOTE — assignee name:** stage-8.5 is assigned to profile `security` (there is no `security-reviewer` profile on this install; `/config/.hermes/profiles/security/` exists with the verified skill set above). Titles keep the role name "security" for clarity.

---

## 5. Monitoring partition (slot-3.5 after rows land)

28 rows → ceil(28/5) = 6 monitor tasks on board `monitoring`, read-only, each: `Board: digital-product-studio / Tasks: <ids> / Root: /config/.hermes/workspace/projects/digital-product-studio/ / Partition: n / heartbeat 5m`.

Partition (no overlap, full coverage):
1. 4.1, 4.2, 4.3, 4.4, 5.1
2. 5.2, 5.3, 5.4, 6.1, 6.2
3. 6.3, 6.4, 7.1, 7.2, 7.3
4. 7.4, 7.5, 7.6, 7.7, 7.8
5. 7.9, 7.10, 8, 8.5, 9
6. 10, 11, 12

*(Partition uses stage-slot labels above; slot-3.5 substitutes the concrete task ids it received from kanban create.)*

---

*End of DAG-schema. Chunk bodies (goal/inputs/outputs/skills/acceptance per row) live in docs/12-chunk-bodies.md; testing strategy in docs/12-testing-strategy.md; the narrative plan in docs/12-implementation-plan.md.*
