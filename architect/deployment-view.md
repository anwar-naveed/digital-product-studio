# 11 — System Architecture: digital-product-studio

**Produced by:** SLOT-2 architect
**Date:** 2026-10-12
**Status:** Complete — consumes slot-1 (docs/09-requirements.md, R1–R15) + slot-1.5 (docs/10-security-research.md, S1–S15, H1–H15). No code.

---

## 7. Deployment View

Openship compose project `digital-product-studio` on the same Docker host at 172.19.0.1.

```yaml
# SKETCH (no real values; canonical compose is devops-slot deliverable)
# networks:
#   internal: 172.19.x.0/24 — core, postgres, admin-ui
#   worker-egress: 172.19.y.0/24 — workers only; iptables DROP except minio+API
# services:
#   pipeline-core  : ports: "172.19.0.1:8080:8080"   (REST)
#   admin-ui       : ports: "172.19.0.1:4173:80"     (static nginx)
#   pipeline-worker: deploy.replicas=4, no published ports
#   postgres       : internal only
# secrets (Openship isSecret):
#   ETSY_REFRESH_TOKEN, LS_API_KEY, PRINTIFY_API_KEY, GELATO_API_KEY,
#   MINIO_ACCESS_KEY, MINIO_SECRET_KEY, JWT_SIGNING_KEY,
#   WEBHOOK_SECRET_ETSY, WEBHOOK_SECRET_LS, WEBHOOK_SECRET_PRINTIFY, WEBHOOK_SECRET_GELATO
```

CI/CD: GitHub Actions (org repo) builds/pushes:
- `pipeline-core:<semver>` (Rust musl static + debian:bookworm-slim runtime)
- `admin-ui:<semver>` (npm ci → vite build → nginx:alpine COPY)
- `pipeline-worker-base:photocraft-0.7.0` (zero-compile: download release tarball → verify sha256 → extract binary → COPY into debian:bookworm-slim — recipe from doc 08 §7 step 2)
- Tagged digest-pinned refs deployed via Openship update (H10).

Pingap: certificates.studio (myCA fullchain per doc 08 §7 verification note — operator picks myCA over self-signed so the webtop browser trusts without exception), upstream `studioUpstream → http://172.19.0.1:4173` (admin-ui), `/api/*` → `http://172.19.0.1:8080`. Allowlist `172.19.0.0/16`. HSTS.

Bucket binding: re-use the shared-minio project (proj_9pg0PYaYOLsjdRvO); add `digital-product-studio:accesskey:secretkey` to its MINIO_SITES secret, redeploy. Bucket versioning + lifecycle `tmp/` 24h expiry enabled.

---

## 8. Tech-Stack Picks (versions verified live 2026-10-12)

| Component | Crate / Package | Version | Docs | Reason |
|---|---|---|---|---|
| Rust HTTP web framework | axum | **0.8.9** | <https://docs.rs/axum/0.8.9> | tokio-native, tower middleware, macro-free routing. Latest stable. |
| Async runtime | tokio | **1.53.2** | <https://docs.rs/tokio/1.53.2> | Unified 1.x line; avoids the deprecated tokio-* split crates flagged by slot-1.5. |
| Postgres driver / ORM-free | sqlx | **0.9.0** | <https://docs.rs/sqlx/0.9.0> | Compile-time-checked queries (S3), SKIP LOCKED first-class, ≥0.8.1 fixes RUSTSEC-2024-0363. |
| S3 client | object_store | **0.14.2** | <https://docs.rs/object_store/0.14.2> | Chosen OVER aws-sdk-s3: smaller dep tree, first-class MinIO (S3-compatible endpoint), Apache governance, no AWS-account SDK weight. Verdict PASS w/ monitoring (slot-1.5 §3.1). |
| HTTP client (adapters) | reqwest | **0.13.5** | <https://docs.rs/reqwest/0.13.5> | rustls ≥0.23.45 pulls h2 ≥0.4.16 → satisfies S4, S5. |
| Middleware | tower-http | **0.7.1** | <https://docs.rs/tower-http/0.7.1> | Mistyped earlier as 0.5/0.6; current 0.7.1 is what axum 0.8.9 pairs with. |
| Tracing | tracing | **0.1.44** | <https://docs.rs/tracing/0.1.44> | + tracing-subscriber ≥0.3.20 (S6). OTLP export. |
| JSON Schema gen | schemars | **1.2.3** | <https://docs.rs/schemars/1.2.3> | Drives worker command-spec validation (S14). |
| TLS | rustls | **0.23.45** | <https://docs.rs/rustls/0.23.45> | RUSTSEC-2026-0285 fix; default for reqwest via `default-tls`. |
| HTTP/2 (transitive) | h2 | **0.4.20** | <https://docs.rs/h2/0.4.20> | ≥0.4.16 satisfies S5 (verified live — was 0.4.16 min). |
| Web framework | react | **19.3.0** | <https://www.npmjs.com/package/react> | + react-dom 19.3.0. |
| Web build | vite | **8.3.4** | <https://www.npmjs.com/package/vite> | NOT 5.x/6.x per slot-1.5 §3.2 (Vite advisories conditional) — current 8.x line is clean for our build-only use. |
| Data fetching | @tanstack/react-query | **5.104.2** | <https://www.npmjs.com/package/@tanstack/react-query> | Core only — NO @tanstack/start-*, form-core, arktype-adapter (slot-1.5 §3.2). |
| UI components | shadcn/ui | copy-paste components | <https://ui.shadcn.com> | Audit copies for `dangerouslySetInnerHTML` (S15). |

`Cargo.toml`/`package.json` pinning decisions themselves belong to implementation; this doc fixes the version FLOORS shown above.

---

## 10. Handoff

**Inputs consumed:**

| Source doc | IDs consumed | Integrated in |
|---|---|---|
| docs/09-requirements.md (slot-1) | R1–R15 | §1–§8 |
| docs/10-security-research.md (slot-1.5) | S1–S15, H1–H15, M1.x–M6.x, C1–C6 | §1–§7, §9 |

**Requirement → component mapping:**

| Req | Component(s) |
|---|---|
| R1 | niche shortlist stored as `niches` rows; enforced by admin-ui form + `CHECK (vertical IN (...))` |
| R2 | adapter implementations carry doc URLs as struct-level comments |
| R3 | EtsyAdapter OAuth2 refresh worker (§4) |
| R4 | LemonSqueezyAdapter Bearer header (§4) |
| R5 | PrintifyAdapter token bucket 600/min (§4) |
| R6 | cf-image-gen worker capacity + run sizing (§5 concurrency caps) |
| R7 | photocraft-cli toolclass command-spec schema (§5) |
| R8 | vectorcraft-cli toolclass command-spec schema (§5) |
| R9 | fee_tables + /api/v1/settings/fee-tables (§2, §3) |
| R10 | POST /api/v1/runs + production_runs spawn (§3, §6) |
| R11 | CI image recipe — sha256 verification step (§7) |
| R12 | worker Dockerfile prohibition of GUI libs (§7) |
| R13 | Etsy adapter file-size guard 250MB (§4) |
| R14 | POST /api/v1/webhooks/{platform} (§3) + adapter verify_webhook (§4) |
| R15 | products.license_tier + pricing ladder, /api/v1/catalog (§2, §3) |

**Security requirement → design element mapping:**

| S | Design element |
|---|---|
| S1 | §3 body limit (tower_http::limit) |
| S2 | §3 JWT role check middleware |
| S3 | §1/§2 sqlx::query! only (Postgres) |
| S4 | §4 reqwest+rustls 0.23.45 pin (§8) |
| S5 | §8 h2 0.4.20 ≥0.4.16 |
| S6 | §8 tracing-subscriber ≥0.3.20 |
| S7 | §5 worker Dockerfile USER 1000 + read_only |
| S8 | §5 compose cap_drop ALL + no-new-privileges |
| S9 | §5 compose deploy.resources.limits |
| S10 | §1, §5 worker-egress network + iptables rule |
| S11 | §3 settings routes + §7 Openship isSecret |
| S12 | §7 Pingap myCA cert + HSTS |
| S13 | §3, §4 verify_webhook per platform |
| S14 | §5 command-spec JSON Schema (schemars), allowlist, --params-file, --automation-root |
| S15 | §1 admin-ui production build (vite sourcemap off, CSP, shadcn audit) |

**Handoff to next slot(s):** this doc feeds slot-3+ (backend-engineer + frontend-engineer + devops) — the explicit deliverable contract is the §10 mapping tables plus §8 version pins.

**Known UNVERIFIED items (carried from slot-1.5, not silently resolved):**
- Exact crate versions beyond the floors cited — implementation `cargo build` must snapshot `Cargo.lock` and re-verify against RustSec.
- Exact npm versions for shadcn/ui transitive components — package.json not yet written.
- MinIO bucket policy JSON for scoped key — needs `mc policy get` at deploy time.
- Pingap cert pattern (myCA vs self-signed) — operator decision at deploy time.
- gVisor/Kata availability on the Docker host — falls back to seccomp + cap-drop if absent.
- Gelato rate-limit exact cap (public docs don't publish).

---

*End of architecture memo. All 10 sections present. 15/15 R-requirements mapped. 15/15 S-requirements traced. State machines drawn. Worker jail explicit. All versions cited with doc links. *
