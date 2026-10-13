# 12 — Chunk Bodies: per-row skeletons (stages 4→12)

**Produced by:** SLOT-3 planner. Every row below carries: goal / inputs / outputs / skills (verified) / acceptance criteria. slot-3.5 pastes these into the kanban row bodies together with the mandatory row-template contract (HARD CLAUSES, MemPalace persist, ROOT_REPO, etc.).

Common to ALL rows:
- ROOT_REPO: `/config/.hermes/workspace/projects/digital-product-studio/`
- MemPalace wing: `digital-product-studio`; room = stage name; drawer(s) named below.
- Docker-only env (4 HARD CLAUSES verbatim from slot-3.5's body template) — code authoring on host OK, builds/tests/installs NEVER on host.
- Research-first: verify any new library/API via web_search/web_extract before use; never assume.
- rg not grep. Never fake-complete; block with exact reason.
- References: docs/09-requirements.md (R1–R15), docs/10-security-research.md (S1–S15, H1–H15), docs/11-architecture.md (§1–§10 + ADR-001..006 + version floors).

---

## Stage 4 — design-ux (design specs; NO code)

### 4.1 design-ux 4.1: Curation grid + production runs (core console UX)
- **Goal:** Wireframes + component hierarchy + interaction spec for the curation grid (approve/reject with reason taxonomy: style mismatch / artifact / license / off-niche / duplicate) and production-runs pages (run list, run detail with job breakdown, spawn-run form: niche pick → target_count → params, cancel).
- **Inputs:** docs/11-architecture.md §6 (curation flow sequence), §2 (job/listing state machines), §3 (routes /runs, /curation/queue, /curation/{id}/approve|reject); MemPalace room=general drawers architecture.md, api-surface.md; MemPalace room=researcher drawer niche-shortlist (niche board feeds run spawn).
- **Outputs:** MemPalace room=design-ux drawers: `4.1-curation-runs-wireframes`, `4.1-curation-runs-component-hierarchy`, `4.1-curation-runs-interactions`. No files.
- **Skills:** design/ui-ux-pro-max; development/react-best-practices (component-naming conventions only).
- **Acceptance criteria:** (a) curation grid spec covers paginated preview grid, presigned-URL image preview, bulk approve/reject, reject-reason taxonomy picker; (b) run pages cover R6 (run sizing) + R10 (niche → run spawn) flows; (c) every screen names the API route it consumes (from §3); (d) component tree ≤3 levels deep per page, reuses ≥1 shared component across pages; (e) drawer ids cited in completion metadata.

### 4.2 design-ux 4.2: Catalog + listings pages
- **Goal:** Wireframes + hierarchy for catalog browser (designs/products/bundles, product detail with items + license tiers R15), publish flow (platform picker → draft → live states), listings management (state machine display draft/publishing/live/failed/suspended/delisted, delist + reprice actions R9/R15).
- **Inputs:** docs/11 §2 listing state machine, §3 routes /catalog, /products/{id}/publish, /listings/{id}/delist|reprice; docs/09 R13 (file limits surfaced as validation hints), R15 (pricing ladder UI).
- **Outputs:** MemPalace room=design-ux drawers: `4.2-catalog-listings-wireframes`, `4.2-catalog-listings-component-hierarchy`.
- **Skills:** design/ui-ux-pro-max.
- **Acceptance criteria:** (a) pricing-ladder editor renders 3 tiers (personal/small-commercial/extended) with per-platform fee preview (fee_tables); (b) listing state badge set matches §2 exactly; (c) publish flow shows per-platform prerequisites (credentials present) from /settings/credentials metadata; (d) drawer ids cited.

### 4.3 design-ux 4.3: Analytics + niche board
- **Goal:** Wireframes for analytics (per-listing performance: views/sales/margin per R9; per-run cost) and niche board (ranked niches with demand_signals + competition_score, "pick niche → spawn run" CTA wired to POST /runs).
- **Inputs:** docs/09 §5 (niche research sources), R9, R10; docs/11 §2 niches/demand_signals schema, §3 /analytics/*, /niches routes.
- **Outputs:** MemPalace room=design-ux drawers: `4.3-analytics-niche-board-wireframes`, `4.3-analytics-niche-board-hierarchy`.
- **Skills:** design/ui-ux-pro-max.
- **Acceptance criteria:** (a) analytics tables define columns for gross/fee/net + Payoneer withdrawal line (R9); (b) niche board cards show demand evidence and spawn-run affordance with target_count input; (c) drawer ids cited.

### 4.4 design-ux 4.4: Settings + auth shell
- **Goal:** Wireframes for settings (credentials metadata list per platform S11 — reference only, never the secret; fee-tables editor; webhook status) and the app shell: nav rail (7 page groups), JWT role=admin auth gate, error/correlation-id toast patterns, dark-theme tokens.
- **Inputs:** docs/10 S11, S15, M3.1 (CSP note); docs/11 §3 settings routes; §1 admin-ui trust zone.
- **Outputs:** MemPalace room=design-ux drawers: `4.4-settings-auth-shell-wireframes`, `4.4-design-tokens-theme`.
- **Skills:** design/ui-ux-pro-max.
- **Acceptance criteria:** (a) credentials page displays ONLY reference metadata (masked, no secret values — S11); (b) shell spec lists the 7 nav sections matching stages 4.1–4.3 page groups + settings; (c) auth gate redirects non-admin to login; (d) drawer ids cited.

---

## Stage 5 — frontend-engineer (implementation design; file names + code snippets, NO repo files yet)

### 5.1 frontend-engineer 5.1: App shell + routing + API client
- **Goal:** Implementation design for admin-ui app shell: React 19.3 + Vite 8.3.4 + TS; route table for all 7 page groups; API client (typed fetch wrapper over /api/v1 with JWT bearer header, error envelope `{error:{code,message,correlation_id}}`, 401→login redirect); file tree + exact file names + code snippets.
- **Inputs:** Stage-4 drawers (all 4.1–4.4 wireframes) from MemPalace room=design-ux; docs/11 §3 (route surface), §8 (version pins).
- **Outputs:** MemPalace room=frontend-engineer drawers: `5.1-app-shell-file-tree`, `5.1-routing-spec`, `5.1-api-client-spec` (with snippets).
- **Skills:** development/react-best-practices; development/frontend-development-patterns; frontend/shadcn-ui.
- **Acceptance criteria:** (a) file tree lists every .tsx/.ts file with one-line purpose; (b) route table maps URL → page component → API routes consumed; (c) API client snippet covers auth header injection, error envelope parse, correlation-id propagation; (d) Vite config snippet sets `build.sourcemap:false` (S15); (e) drawer ids cited.

### 5.2 frontend-engineer 5.2: State/data layer (TanStack Query hooks per domain)
- **Goal:** Design TanStack Query (@tanstack/react-query 5.104.2, CORE ONLY) hooks per domain: useNiches, useRuns, useCuration, useCatalog, useProducts, useListings, useAnalytics, useSettings — query keys, invalidation graph, mutation wrappers (POST /runs, approve/reject, publish, delist, reprice, fee-tables PUT).
- **Inputs:** docs/11 §3 routes; stage-4 drawers; 5.1 drawers (client shape).
- **Outputs:** MemPalace room=frontend-engineer drawers: `5.2-query-hooks-spec`, `5.2-invalidation-map`.
- **Skills:** development/react-best-practices; development/testing-and-tdd (hook test plan).
- **Acceptance criteria:** (a) every §3 route has a hook or mutation; (b) invalidation map states exactly which queries refetch after each mutation (e.g. approve → curation queue + catalog); (c) no @tanstack/start-*, form-core, arktype-adapter (slot-1.5 §3.2); (d) drawer ids cited.

### 5.3 frontend-engineer 5.3: Curation + runs UI components
- **Goal:** Component specs (props, state, snippets) for: CurationGrid, CurationCard (presigned preview + approve/reject + reason picker), RunList, RunDetail (job breakdown table + progress), SpawnRunDialog; composition per 4.1 wireframes.
- **Inputs:** 4.1 drawers; 5.2 hook specs; docs/11 §6 sequence (approve → S3 COPY + assembly job).
- **Outputs:** MemPalace room=frontend-engineer drawer: `5.3-curation-runs-components-spec`.
- **Skills:** development/react-best-practices; frontend/shadcn-ui; development/testing-and-tdd.
- **Acceptance criteria:** (a) each component: props table + min code snippet + consumed hook; (b) reject-reason picker options match the 5-value taxonomy; (c) RunDetail renders job states queued/claimed/running/done/failed/dead with attempts + last_error; (d) drawer id cited.

### 5.4 frontend-engineer 5.4: Catalog + listings + analytics components
- **Goal:** Component specs for CatalogBrowser, ProductDetail (license tiers + publish CTA), PublishDialog (platform multi-select), ListingsTable (state badges + delist/reprice actions), AnalyticsTables (performance + run-cost), NicheBoard (cards + spawn CTA), SettingsPages (credentials metadata, fee-tables editor).
- **Inputs:** 4.2/4.3/4.4 drawers; 5.2 hook specs; docs/11 §2 listing states, §3 routes.
- **Outputs:** MemPalace room=frontend-engineer drawer: `5.4-catalog-listings-analytics-components-spec`.
- **Skills:** development/react-best-practices; frontend/shadcn-ui.
- **Acceptance criteria:** (a) listing state badge set matches §2 exactly (draft/publishing/live/updating/failed/suspended/delisted); (b) pricing ladder editor binds 3 tiers (R15) with fee preview (R9); (c) NicheBoard CTA posts {niche_id, target_count, params} per §3; (d) drawer id cited.

---

## Stage 6 — backend-engineer (implementation design; file names + code snippets, NO repo files yet)

### 6.1 backend-engineer 6.1: pipeline-core service shape
- **Goal:** Crate layout (workspace members: pipeline-core, worker, adapters), axum 0.8.9 app bootstrap, config loader (env-driven, no secrets in code — S11), sqlx 0.9.0 PgPool setup, migrations framework choice (sqlx migrate), tracing 0.1.44 + subscriber ≥0.3.20 init (S6), error envelope type, middleware stack (tower-http 0.7.1: body limit 2MB S1, timeout, trace), JWT role=admin middleware (S2), audit-event emit helper (M1.3). Exact Cargo.toml pinned to §8 floors + file tree + snippets.
- **Inputs:** docs/11 §1 (components), §3 (API surface), §8 (version pins), §9 ADR-001/002; docs/10 S1–S6, M1.x.
- **Outputs:** MemPalace room=backend-engineer drawers: `6.1-crate-layout-cargo`, `6.1-app-bootstrap-spec`, `6.1-middleware-auth-spec`.
- **Skills:** development/backend-development-patterns; development/universal-development-standards.
- **Acceptance criteria:** (a) file tree lists every module path under pipeline-core/src, worker/src, adapters/src; (b) Cargo.toml snippet pins axum 0.8.9, tokio 1.53.2, sqlx 0.9.0, tower-http 0.7.1, tracing 0.1.44, reqwest 0.13.5, rustls 0.23.45, object_store 0.14.2, schemars 1.2.3 (all floors per §8); (c) middleware spec shows 2MB limit + JWT check + audit emission on mutating routes; (d) migration framework decision (sqlx migrate) stated with rationale; (e) drawer ids cited.

### 6.2 backend-engineer 6.2: Job queue + worker runner + command-spec jail
- **Goal:** Spec for: jobs table claim query (SELECT … FOR UPDATE SKIP LOCKED, next_attempt_at filter, 120s lease), reaper loop, retry/backoff (30s*2^attempts, cap 3), worker pull loop (claim → exec → receipt), JobCommandSpec schema (schemars-derived, per §5 JSON) with per-tool args schemas, injection jail implementation spec (--params-file, Vec<String> args, canonicalize under /workspace/{read,write}, subcommand allowlist), execution receipt shape → job_receipts.
- **Inputs:** docs/11 §5 (worker model), §2 job state machine; docs/10 S14, M2.1–M2.6, command-injection deep dive; 6.1 drawers.
- **Outputs:** MemPalace room=backend-engineer drawers: `6.2-queue-schema-queries`, `6.2-worker-loop-spec`, `6.2-command-spec-jail-spec`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance criteria:** (a) claim query is one atomic txn with lease fields; (b) jail spec covers all 4 injection vectors (params, out-path, in-path, subcommand) with mitigations; (c) receipt JSON shape matches M2.3; (d) per-tool (cf-image-gen, photocraft-cli, vectorcraft-cli) args-schema approach defined via schemars 1.2.3; (e) drawer ids cited.

### 6.3 backend-engineer 6.3: Platform adapters
- **Goal:** PlatformAdapter trait (per §4) + per-adapter design: EtsyAdapter (OAuth2 + PKCE + proactive refresh M5.6, token bucket 5rps/10burst, HMAC-SHA256 verify, 250MB/format guard R13), LemonSqueezyAdapter (Bearer, 150rpm bucket, X-Signature), PrintifyAdapter (Bearer, 600/min firm R5, 8rps/20burst), GelatoAdapter (X-API-KEY, 4rps jittered); shared infra: reqwest client with rustls, TokenBucket layer, RetryPolicy (exp backoff + jitter, honor Retry-After, max 5, on 429/5xx); webhook ingestion design (verify-before-any-DB-write S13, replay cache processed_webhook_ids TTL 7d M6.2); rate-limit/secret plumbing (secrets from Openship isSecret env).
- **Inputs:** docs/09 §2 (platform matrix R3–R5, R13, R14); docs/11 §4 (trait + table); docs/10 S4, S13, M5.x, M6.x.
- **Outputs:** MemPalace room=backend-engineer drawers: `6.3-adapter-trait-spec`, `6.3-per-adapter-designs`, `6.3-webhook-ingestion-spec`.
- **Skills:** development/backend-development-patterns; development/universal-development-standards.
- **Acceptance criteria:** (a) trait signature matches §4 verbatim (6 methods + rate_limiter); (b) each adapter: auth flow, rate strategy, webhook verify + signature header name, upload flow steps citing doc URLs (R2 — URLs carried from docs/09 §2 tables); (c) verify_webhook is constant-time and runs before persistence; (d) drawer ids cited.

### 6.4 backend-engineer 6.4: Admin REST API surface + S3 client layer
- **Goal:** Handler design for every §3 route (23 routes): request/response DTOs, validation (zod-equivalent via serde + schemars), audit event on each mutation, pagination for curation queue + catalog + listings; S3 layer via object_store 0.14.2: bucket config, key builder for the §1 prefix scheme (raw/curated/products/mockups/listings/tmp), presigned GET for previews, COPY raw→curated on approve, lifecycle notes (tmp/ 24h); admin-api auth wiring (JWT role=admin; webhook route exempt but signature-verified).
- **Inputs:** docs/11 §1 (S3 key scheme), §3 (all routes), §6 (curation flow steps); docs/10 S1, S2, S11.
- **Outputs:** MemPalace room=backend-engineer drawers: `6.4-admin-api-handler-spec`, `6.4-dto-schema-spec`, `6.4-s3-layer-spec`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance criteria:** (a) all 23 §3 routes specified with method, path, DTO, audit, R/S trace; (b) S3 key builder maps every lifecycle prefix; (c) presign + copy flows match §6 sequence; (d) drawer ids cited.

---

## Stage 7 — coder (real files under project root; TDD tests written with module)

**Common coder-row contract:** write code under `/config/.hermes/workspace/projects/digital-product-studio/` per 6.x/5.x specs; every module lands WITH its Rust unit tests (`#[cfg(test)]`) or vitest specs; NO host builds (cargo/npm only inside throwaway Docker via devops-provided image or official base); persist summary + file list to MemPalace room=coder drawer; cite upstream spec drawer ids.

### 7.1 coder 7.1: db schema + migrations + entities
- **Goal:** sqlx migrations (all 12 tables per §2: niches, production_runs, jobs, job_receipts, designs, design_variants, curations, products, product_items, platform_listings, sales_events, fee_tables, listing_states, processed_webhook_ids cache) + Rust entity structs + query modules.
- **Inputs:** 6.1 drawers (crate layout, migrations framework); docs/11 §2 (ERD, state machines, CHECK constraints from R1 vertical list).
- **Outputs:** files `migrations/*.sql`, `pipeline-core/src/db/{mod,entities,queries}.rs`; MemPalace room=coder drawer `7.1-db-schema-delivery`.
- **Skills:** development/backend-development-patterns; development/universal-development-standards; development/testing-and-tdd.
- **Acceptance:** (a) every §2 table present with FK edges; (b) job/listing state CHECK constraints match state machines; (c) niches.vertical CHECK IN (3 launch verticals) R1; (d) compile-time sqlx::query! macros only (S3); (e) unit tests for entity serde round-trip.

### 7.2 coder 7.2: job queue + worker loop
- **Goal:** claim/reaper/retry queries (SKIP LOCKED, lease, backoff), worker pull loop binary (worker crate), container-spawn executor interface, heartbeat.
- **Inputs:** 6.2 drawers; 7.1 (entities).
- **Outputs:** files `pipeline-core/src/queue/*.rs`, `worker/src/{main,loop,exec}.rs`; MemPalace drawer `7.2-queue-worker-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) claim = single txn, no double-claim (unit test with two concurrent claims inside Docker pg); (b) reaper expires stale running jobs; (c) backoff matches 30s*2^attempts cap 3; (d) integration test: enqueue → claim → complete transitions queued→claimed→running→done.

### 7.3 coder 7.3: command-spec validators + S3 staging
- **Goal:** JobCommandSpec types + schemars JSON schemas for the 3 tools, subcommand allowlist, params-file writer, path canonicalization jail, expected-outputs verify, S3 staging helpers (object_store PUT/GET/presign/COPY for §1 prefixes).
- **Inputs:** 6.2 jail spec + 6.4 S3 layer spec; docs/10 S14; docs/11 §5 schema.
- **Outputs:** files `pipeline-core/src/jobspec/*.rs`, `pipeline-core/src/s3/*.rs`; MemPalace drawer `7.3-jailspec-s3-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd; development/universal-development-standards.
- **Acceptance:** (a) fuzz-style unit test: adversarial specs (traversal `../../`, unknown subcommand, shell metachars, symlink) ALL rejected — S14; (b) valid spec → args vector built with zero shell interpolation; (c) S3 key builder unit tests per prefix; (d) presign/COPY helpers unit-tested against MinIO testcontainer inside Docker.

### 7.4 coder 7.4: etsy adapter
- **Goal:** EtsyAdapter impl: OAuth2 PKCE + refresh worker, publish/update/upload-file (≤250MB + format guard R13), fetch_orders/fetch_stats, verify_webhook HMAC, token bucket 5/10.
- **Inputs:** 6.3 drawers (trait + etsy design); 7.3 (S3 for file staging); docs/09 §2.1 URLs (as code comments — R2).
- **Outputs:** files `adapters/src/{platform.rs,etsy/*.rs}`; MemPalace drawer `7.4-etsy-adapter-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) contract tests against mock Etsy server (in-Docker wiremock): happy publish, 429+Retry-After backoff, invalid-sig webhook 401, oversize file 4xx; (b) refresh fires ≥5min pre-expiry (M5.6); (c) doc-URL comments on each API call site.

### 7.5 coder 7.5: lemonsqueezy adapter
- **Goal:** LemonSqueezyAdapter impl: Bearer, product/checkout flows, license-key webhooks, X-Signature verify, 150rpm bucket.
- **Inputs:** 6.3 drawers; docs/09 §2.2 URLs.
- **Outputs:** files `adapters/src/lemonsqueezy/*.rs`; MemPalace drawer `7.5-ls-adapter-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) mock-server contract tests: no-auth 401 path, signed-webhook accept, tampered-sig reject; (b) R4 unit test (request without Bearer → 401); (c) doc-URL comments.

### 7.6 coder 7.6: printify + gelato adapters
- **Goal:** PrintifyAdapter (Bearer, 600/min firm — R5 test at 601st request, HMAC webhooks) + GelatoAdapter (X-API-KEY, 4rps jitter, conservative cap per slot-1.5 UNVERIFIED note).
- **Inputs:** 6.3 drawers; docs/09 §2.3–2.4 URLs; docs/11 §4 table.
- **Outputs:** files `adapters/src/{printify,gelato}/*.rs`; MemPalace drawer `7.6-pod-adapters-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) R5 reproduced in mock test (bucket refuses 601st within window, backoff succeeds); (b) gelato jittered backoff unit test; (c) webhooks verified both; (d) doc-URL comments.

### 7.7 coder 7.7: admin API niches + runs + curation
- **Goal:** Handlers + DTOs: niches CRUD (cascade guard), runs (spawn inserts run + N cf-image-gen jobs per §6 txn, list, detail, cancel), curation queue (paginated presigned previews), approve (curations row + S3 COPY + assembly-job enqueue), reject (reason taxonomy).
- **Inputs:** 6.4 handler/DTO drawers; 7.1 (queries), 7.3 (S3 COPY); docs/11 §3, §6.
- **Outputs:** files `pipeline-core/src/api/{niches,runs,curation}.rs`; MemPalace drawer `7.7-admin-api-core-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) route tests in-Docker: POST /runs creates run + target_count jobs atomically (R10); (b) approve performs COPY + assembly enqueue in one flow (§6 steps 8–11); (c) all mutations emit audit events (M1.3) — asserted in tests; (d) 2MB body limit 413 test (S1) on one representative route.

### 7.8 coder 7.8: admin API catalog + listings + analytics
- **Goal:** Handlers: catalog browse, product detail, publish (enqueue per-platform publish jobs), delist, reprice (pricing ladder R15 + fee calc R9), analytics performance + run cost (gross/fee/net + Payoneer line), settings credentials (metadata only S11) + fee-tables PUT.
- **Inputs:** 6.4 drawers; 7.4–7.6 (adapters for publish jobs); docs/11 §3.
- **Outputs:** files `pipeline-core/src/api/{catalog,listings,analytics,settings}.rs`; MemPalace drawer `7.8-admin-api-rest-delivery`.
- **Skills:** development/backend-development-patterns; development/testing-and-tdd.
- **Acceptance:** (a) publish enqueues platform jobs with correct adapter routing; (b) reprice computes 3-tier ladder + fee preview per fee_tables (R9/R15 test); (c) credentials endpoint NEVER returns secret values (S11 test asserts); (d) analytics join returns per-listing views/sales/margin.

### 7.9 coder 7.9: admin-ui shell + pages
- **Goal:** From 5.1–5.2 specs: Vite+React+TS scaffold (files only, no host npm install), router, layout shell, API client, all query hooks, page components for 7 groups wired to 7.7/7.8 routes.
- **Inputs:** 5.1–5.4 drawers; 6.4 DTO schemas (types must match); 7.7/7.8 route list.
- **Outputs:** files `admin-ui/{package.json,vite.config.ts,src/**}`; MemPalace drawer `7.9-adminui-shell-delivery`.
- **Skills:** development/react-best-practices; frontend/shadcn-ui; development/universal-development-standards; development/testing-and-tdd.
- **Acceptance:** (a) file tree matches 5.1 spec exactly (every named file exists); (b) vite config sourcemap:false (S15); (c) package.json pins react 19.3.0, vite 8.3.4, @tanstack/react-query 5.104.2 core only; (d) vitest specs for hooks + one page render (run inside Docker); (e) no dangerouslySetInnerHTML anywhere (S15 grep-clean).

### 7.10 coder 7.10: admin-ui components
- **Goal:** From 5.3–5.4 specs: CurationGrid, CurationCard, RunList, RunDetail, SpawnRunDialog, CatalogBrowser, ProductDetail, PublishDialog, ListingsTable, AnalyticsTables, NicheBoard, SettingsPages + shared primitives (badge/table/dialog).
- **Inputs:** 5.3–5.4 drawers; 7.9 (hooks/pages to compose into).
- **Outputs:** files `admin-ui/src/components/**`; MemPalace drawer `7.10-adminui-components-delivery`.
- **Skills:** development/react-best-practices; frontend/shadcn-ui; development/testing-and-tdd.
- **Acceptance:** (a) every 5.3/5.4 component exists with props matching spec; (b) reject-reason picker options = 5-value taxonomy; (c) listing badges cover all 8 states; (d) vitest component tests (Docker) for CurationCard approve/reject + ListingsTable badge mapping.

---

## Stages 8→12 — single rows

### 8 reviewer 8: verify implementation vs architect decisions
- **Goal:** Review all 7.x deliverables against docs/11 (§1–§10, ADRs): route/DTO/route-coverage parity, state-machine fidelity, S3 prefix fidelity, trait conformance, version pins in Cargo.toml/package.json vs §8 floors.
- **Inputs:** repo tree (read-only), MemPalace rooms backend-engineer/frontend-engineer/coder drawers, docs/11.
- **Outputs:** MemPalace room=reviewer drawers: `8-review-findings`, `8-review-verdict` (approve | request-changes with per-file list); comment on the stage-8 kanban row.
- **Skills:** software-development/code-analysis; software-development/requesting-code-review; development/universal-development-standards.
- **Acceptance:** (a) parity checklist for all 23 routes + 12 tables + 4 adapters filed; (b) every deviation listed with file:line; (c) verdict is binary + actionable; (d) NO code edits (review-only).

### 8.5 security 8.5: verify vs S1–S15
- **Goal:** Independently verify each S1–S15 testable criterion against the delivered code + devops artifacts (S7–S12 need stage-9 outputs where applicable; stage-8.5 verifies code-side S1–S6, S11, S13–S15 and files an S7–S12 checklist for devops).
- **Inputs:** docs/10 §5 (S1–S15), §4 (H1–H15); repo; MemPalace room=reviewer verdict.
- **Outputs:** MemPalace room=security drawers: `8.5-s-verification-matrix` (15 rows: pass/fail + evidence file:line), `8.5-blockers`.
- **Skills:** software-development/code-analysis; software-development/systematic-debugging; development-rules.
- **Acceptance:** (a) all 15 S-items have pass/fail + evidence pointer; (b) jail fuzz suite (S14) re-run inside Docker, 0 injections; (c) failures block (stage-9 does NOT start until 8.5 passes or waiver noted); (d) NO code edits.

### 9 devops 9: Dockerfiles + compose + GH Actions + Openship + Pingap/cert/bucket
- **Goal:** Author (files only; never build on host): pipeline-core Dockerfile (multi-stage cargo-chef → musl? NO — debian runtime per §7 sketch), admin-ui Dockerfile (node build → nginx:alpine COPY), pipeline-worker-base Dockerfile (download photocraft 0.7.0 + vectorcraft 0.9.0 tarballs → sha256 verify R11 → extract → bookworm-slim), docker-compose.yml (4 services + 2 networks + secrets + H1–H12 hardening: UID 1000, read_only, cap_drop, no-new-privileges, resource limits, worker-egress), .github/workflows/build.yml (build+push all 3 images, digest-pinned, trivy H13, cargo audit + npm audit H14), Openship project prep (compose project digital-product-studio, ports on 172.19.0.1, isSecret list), Pingap cert + host config (studio.amzankos.local, myCA fullchain, allowlist 172.19.0.0/16, HSTS, /api/* → 8080, / → 4173), MinIO bucket steps (scoped key to bucket + versioning + tmp/ 24h lifecycle per §7).
- **Inputs:** docs/10 §4 (H1–H15), docs/11 §7 (deployment view + sketch), docs/08 §7 (tarball recipe), 8.5 verdict.
- **Outputs:** files `deploy/{Dockerfile.core,Dockerfile.ui,Dockerfile.worker,docker-compose.yml}`, `.github/workflows/build.yml`, `deploy/{pingap,openship,minio}.md` step docs; MemPalace room=devops drawer `9-deployment-delivery`.
- **Skills:** devops/docker-expert; devops/openship-deployment; devops/docker-nginx-shared-edge; devops/monitoring-stack-integration; github/github-repo-management.
- **Acceptance:** (a) compose encodes H1–H12 verbatim (inspectable keys); (b) workflow never references local runners/exec; (c) worker Dockerfile verifies SHA256SUMS (R11) and adds no GUI libs (R12 — apt install list grep-clean of x11/wayland); (d) secrets ONLY via isSecret (S11 — compose grep for literal keys returns placeholders only); (e) Pingap + bucket step docs are operator-runnable scripts with verify commands (S12 openssl probe, mc policy get).

### 10 tester 10: test plan + run inside Docker
- **Goal:** Consolidated test plan + EXECUTE all tests inside throwaway containers (CI images or official bases; never host): cargo unit+integration (queue, jail fuzz S14, adapters contract via mock servers, admin API routes), admin-ui vitest + build. Gate: everything green.
- **Inputs:** 9 deployment artifacts; repo test suites from 7.x; docs/12-testing-strategy.md.
- **Outputs:** MemPalace room=tester drawers: `10-test-plan`, `10-test-results` (per-suite pass/fail counts + container ids/commands), `10-failures`; comment on stage-10 row.
- **Skills:** software-development/test-driven-development; development/testing-and-tdd; software-development/code-analysis.
- **Acceptance:** (a) every suite from 7.x executed in-container with captured output; (b) S1/S13/S14 security tests re-run and green; (c) failures block stage-11 with exact failing test ids; (d) host-cleanliness probe (HARD CLAUSE 4) output attached.

### 11 qa-engineer 11: e2e smoke
- **Goal:** Black-box e2e against the deployed compose stack (devops brings it up in Docker): spawn run (niche=wall-art gothic, target_count=2) → worker generates placeholder images (cf-image-gen stub in test mode) → curate approve 1/reject 1 → product assembly fires → publish-dry-run to mock platform adapters (no real API calls) → listing reaches `publishing→live(dry)` state; verify S3 objects land in MinIO prefixes.
- **Inputs:** running stack from 9/10; mock-mode adapter flags (from 6.3 spec); docs/11 §6 sequence.
- **Outputs:** MemPalace room=qa-engineer drawers: `11-e2e-smoke-script`, `11-e2e-results` (step-by-step pass/fail with correlation ids); comment on stage-11 row.
- **Skills:** software-development/dogfood; software-development/test-driven-development; software-development/systematic-debugging.
- **Acceptance:** (a) full §6 happy-path traversed with real HTTP calls to studio.amzankos.local (via webtop host entry); (b) each step's API response + DB state + S3 key asserted; (c) publish-dry-run makes ZERO outbound platform calls (egress net asserts); (d) failures block stage-12.

### 12 documenter 12: README + ops runbook
- **Goal:** docs/README.md rewrite (project overview, quickstart via compose, architecture pointers to docs/11) + docs/OPERATIONS.md runbook (deploy via Openship update, secret rotation H15/INCIDENT_RESPONSE.md stub, worker scaling, queue monitoring queries, Pingap cert renewal, MinIO key rotation, backup/restore of Postgres).
- **Inputs:** all prior drawers (esp. 9 deploy docs, 10 test commands, 11 e2e script); docs/08–12.
- **Outputs:** files `docs/README.md`, `docs/OPERATIONS.md`, `docs/INCIDENT_RESPONSE.md` stub; MemPalace room=documenter drawer `12-docs-delivery`.
- **Skills:** document-driven-implementation; development-rules; github/github-repo-management.
- **Acceptance:** (a) runbook commands are copy-paste runnable and reference only Docker/Openship surfaces (host stays clean); (b) every §3 route + every compose secret documented; (c) incident runbook covers the 4 H15 scenarios; (d) no secrets or real endpoints in docs.

---

*End of chunk bodies. Sequencing + skills list + monitoring partition: docs/12-dag-schema.md. Testing strategy: docs/12-testing-strategy.md.*
