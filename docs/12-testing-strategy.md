# 12 — Testing Strategy: digital-product-studio

**Produced by:** SLOT-3 planner. Governs stages 7 (TDD with module) → 8/8.5 (verification) → 10 (execution) → 11 (e2e). All runs INSIDE Docker (HARD CLAUSE 3); host stays clean (HARD CLAUSE 4 probe after every suite).

---

## 1. Principles

1. **TDD in coder chunks (stage 7):** every 7.x chunk lands its module WITH tests — RED→GREEN→REFACTOR inside the chunk; tests run in-container before chunk completion.
2. **No host execution:** cargo test / vitest / integration run only in throwaway containers (CI-built image from stage 9, or official bases: rust:1-bookworm, node:26-bookworm-slim, postgres:16, wiremock, local minio). `--rm` always.
3. **Security tests are first-class:** S1–S15 each have an executable test (docs/10 §5); stage 8.5 re-runs the code-side set; stage 10 re-runs ALL.
4. **Mock platforms, not real APIs:** every adapter test runs against an in-container mock HTTP server (wiremock or axum-test) — zero external egress in CI/tests; stage 11 publish-dry-run asserts egress == 0.

## 2. Per-module test matrix

| Module (chunk) | Test type | Framework | What is asserted | Docker vehicle |
|---|---|---|---|---|
| 7.1 db schema | unit + migration | sqlx::query! compile checks; pgtest | 12 tables, FKs, CHECK constraints (vertical IN 3 niches R1; job/listing states per §2); serde round-trip entities | postgres:16 container + test runner |
| 7.2 queue/worker | integration | tokio tests + real pg | atomic claim (2 concurrent claimers → 1 winner), lease reaper, backoff 30s·2^n cap 3, queued→claimed→running→done | rust image + postgres:16 |
| 7.3 jail + S3 | unit + fuzz | schemars validation loops (10k adversarial specs, S14) + object_store against testcontainer MinIO | traversal/`../` rejected, unknown subcommand rejected, shell metachars rejected, symlink escape rejected, params-file used (never shell), S3 key builder per prefix, presign/COPY | rust + minio testcontainer |
| 7.4 etsy | contract (mock server) | wiremock/axum mock | OAuth refresh ≥5min pre-expiry, publish happy path, 429 + Retry-After honored, HMAC verify ok/tamper→401, 250MB guard, format guard | rust + mock container |
| 7.5 lemonsqueezy | contract | mock server | no-Bearer→401 (R4), signed webhook accept, tampered reject, product/checkout payloads | rust + mock |
| 7.6 printify+gelato | contract | mock server | R5: 601st request in window refused then backoff succeeds; gelato jittered backoff; webhook HMAC both | rust + mock |
| 7.7 admin API core | integration | axum in-process tests + real pg | POST /runs atomic run+N jobs (R10), approve = COPY + assembly enqueue (§6), reject taxonomy values, audit event emitted per mutation (M1.3), S1 413 on 3MB body, S2 401/403 on missing/role!=admin | rust + postgres |
| 7.8 admin API rest | integration | same | publish enqueues platform jobs, reprice 3-tier ladder + fee preview (R15/R9), credentials endpoint leaks no secret (S11), analytics joins | rust + postgres |
| 7.9 admin-ui shell | unit + build | vitest + vite build | file-tree parity, hook tests, one page render, sourcemap:false (S15), package.json pins, zero dangerouslySetInnerHTML (S15) | node:26 container |
| 7.10 admin-ui components | unit (component) | vitest + testing-library | CurationCard approve/reject callbacks + reason options, ListingsTable 8 state badges, SpawnRunDialog validation | node:26 container |

## 3. Integration + security suites (stage 10 executes; stage 8.5 spot-verifies)

1. **Queue integration:** full lifecycle incl. crash simulation (kill worker mid-run → lease expiry → requeue → done).
2. **Jail fuzz (S14):** 10,000 random/adversarial job specs → 0 injections, 0 escapes, 0 unknown subcommands (exit criterion from docs/10).
3. **Webhook suite (S13):** per platform: valid sig→200, invalid→401, missing header→400, replay→200-no-op (idempotency cache).
4. **Body limit (S1):** 3MB POST → 413 on representative mutating routes.
5. **Auth matrix (S2):** no token→401, role=user→403, role=admin→200 across all mutating routes.
6. **Dependency floors (S4–S6):** `cargo tree` asserts rustls ≥0.23.45, h2 ≥0.4.16, tracing-subscriber ≥0.3.20 inside the build container.
7. **Secret hygiene (S11):** compose/Dockerfile/code grep for literal secrets → placeholders only.

## 4. Stage-10 execution contract

- Tester re-runs EVERY suite from §2/§3 in fresh containers; records per-suite: command, container image+id, pass/fail counts, output snippet.
- Any red → block stage-11 with exact failing test ids; no partial passes.
- Completes with host-cleanliness probe (HARD CLAUSE 4: `which cargo rustc node npm` empty; no node_modules/; no target/).

## 5. Stage-11 e2e smoke (black-box, deployed stack)

Flow (docs/11 §6): spawn run (gothic wall-art, target 2) → worker produces placeholder PNGs → curation grid shows 2 pending → approve 1 + reject 1 (reason: artifact) → assembly job fires → product + draft listings appear → publish-dry-run (mock adapters) → listing state publishing→live(dry). Asserts per step: HTTP response, DB row state, S3 object key under correct prefix (raw/ → curated/ → products/), audit log row exists. Egress net-assert: zero external platform calls. Failure at any step blocks stage-12 with correlation id.

## 6. Quality gates (traceability)

- Stage 8 (reviewer): parity checklist 23 routes / 12 tables / 4 adapters / 8 listing states / §8 version pins.
- Stage 8.5 (security): 15-row S-matrix with file:line evidence; S7–S12 items verified at stage 9/10 where they are deploy-side.
- Stage 11 (qa): e2e green + egress-zero proof.
- Every gate files its evidence drawers to MemPalace (room=reviewer/security/qa-engineer) — no gate passes on self-report alone.

---

*End of testing strategy.*
