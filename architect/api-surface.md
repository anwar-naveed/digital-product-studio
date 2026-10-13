# 11 — System Architecture: digital-product-studio

**Produced by:** SLOT-2 architect
**Date:** 2026-10-12
**Status:** Complete — consumes slot-1 (docs/09-requirements.md, R1–R15) + slot-1.5 (docs/10-security-research.md, S1–S15, H1–H15). No code.

---

## 3. API Surface (REST, /api/v1)

All responses JSON. Errors shaped `{error: {code, message, correlation_id}}`. All mutating routes require JWT `role=admin` and emit audit events (M1.3). Body limit 2MB (S1).

| Method | Route | Purpose | R/S trace |
|---|---|---|---|
| GET | /api/v1/health | liveness | — |
| GET | /api/v1/niches | list niches w/ demand scores | R10 |
| POST | /api/v1/niches | create niche | R10 |
| GET | /api/v1/niches/{id} | detail | — |
| PATCH | /api/v1/niches/{id} | edit | — |
| DELETE | /api/v1/niches/{id} | delete (cascade guard if runs exist) | — |
| POST | /api/v1/runs | spawn production run (niche_id, target_count, params) | R6, R10 |
| GET | /api/v1/runs | list with status/progress | — |
| GET | /api/v1/runs/{id} | detail, job breakdown | — |
| POST | /api/v1/runs/{id}/cancel | soft cancel (drains queued; running leases expire) | — |
| GET | /api/v1/curation/queue | paginated previews of decided=pending | curation flow §6 |
| POST | /api/v1/curation/{variant_id}/approve | approve → curated S3 copy job | — |
| POST | /api/v1/curation/{variant_id}/reject | reject with reason | — |
| GET | /api/v1/catalog | browse designs/products/bundles | R15 |
| GET | /api/v1/products/{id} | detail with listings | — |
| POST | /api/v1/products/{id}/publish | enqueue publish to platform(s) | R3,R4 |
| POST | /api/v1/listings/{id}/delist | retire | — |
| POST | /api/v1/listings/{id}/reprice | apply pricing ladder | R9,R15 |
| GET | /api/v1/analytics/performance | per-listing views/sales/margin | R9 |
| GET | /api/v1/analytics/runs | cost per run | — |
| GET | /api/v1/settings/credentials | list credential refs (metadata only) | S11 |
| PUT | /api/v1/settings/credentials/{platform} | store Openship isSecret ref (NOT the secret itself) | S11 |
| GET | /api/v1/settings/fee-tables | list | R9 |
| PUT | /api/v1/settings/fee-tables/{platform} | update fee rules | R9 |
| POST | /api/v1/webhooks/{platform} | webhook ingress (unauth but signature-verified) | S13, R14 |

Webhook ingress policy: no authentication header, but signature verification (S13) is performed BEFORE any DB write. Replay protection via `processed_webhook_ids` cache (M6.2).

---

## 4. Adapter Trait Design

```text
trait PlatformAdapter {
    async fn publish_listing(&self, product: &Product, files: &[UploadFile])
        -> Result<PublishReceipt>;
    async fn update_listing(&self, listing_id: &str, patch: ListingPatch)
        -> Result<()>;
    async fn upload_digital_file(&self, listing_id: &str, file: UploadFile)
        -> Result<PlatformFileId>;
    async fn fetch_orders(&self, since: DateTime) -> Result<Vec<Order>>;
    async fn fetch_stats(&self, listing_id: &str) -> Result<ListingStats>;
    fn verify_webhook(&self, body: &[u8], headers: &HeaderMap)
        -> Result<VerifiedWebhook, VerificationError>;
    fn rate_limiter(&self) -> &RateLimiter;   // token bucket per adapter
}
```

| Platform | Auth | Rate-limit strategy | Webhook signature |
|---|---|---|---|
| **Etsy v3** | OAuth2 `authorization_code` w/ PKCE; refresh token (90d); proactive refresh ≥5 min before expiry (M5.6); access token 1h | Token bucket: 5 req/s sustained, 10 burst (well under Etsy 1000/hr); exponential backoff on 429 with `Retry-After` honored | HMAC-SHA256 over body using webhook secret |
| **Lemon Squeezy** | Bearer API key; rotation manual via admin console | 150 req/min bucket (assumed conservative; UNVERIFIED — public docs don't publish) | `X-Signature` HMAC-SHA256 |
| **Printify** | `Authorization: Bearer <api_key>`; account-scoped | 600 req/min firm cap (R5); bucket 8 req/s with burst 20 | Webhook HMAC |
| **Gelato** | API key (`X-API-KEY`); some flows OAuth2 | Conservative 4 req/s + jittered backoff; UNVERIFIED public cap | HMAC signature |

Common adapter infrastructure: shared HTTP client (reqwest w/ rustls ≥0.23.45, S4), shared `TokenBucket` (tower Layer), shared `RetryPolicy` (exp backoff + jitter, max 5 attempts, only on 429/5xx).

---

