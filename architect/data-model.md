# 11 — System Architecture: digital-product-studio

**Produced by:** SLOT-2 architect
**Date:** 2026-10-12
**Status:** Complete — consumes slot-1 (docs/09-requirements.md, R1–R15) + slot-1.5 (docs/10-security-research.md, S1–S15, H1–H15). No code.

---

## 2. Data Model

All tables in Postgres 16. PKs are `uuid` (gen_random_uuid()). `timestamptz` everywhere. JSONB only for truly schemaless payloads (command_spec, platform_raw).

```mermaid
erDiagram
    niches ||--o{ production_runs : spawns
    production_runs ||--o{ jobs : contains
    jobs ||--o| job_receipts : produces
    production_runs ||--o{ designs : yields
    designs ||--o{ design_variants : has
    design_variants ||--o{ curations : reviewed_in
    designs ||--o{ products : assembled_into
    products ||--o{ product_items : contains
    design_variants ||--o{ product_items : included_in
    products ||--o{ platform_listings : published_as
    platform_listings ||--o{ sales_events : tracks
    platform_listings ||--o{ listing_states : history
    fee_tables ||--o{ platform_listings : priced_by

    niches {
        uuid id PK
        text name
        text vertical
        jsonb demand_signals
        numeric competition_score
        timestamptz created_at
    }
    production_runs {
        uuid id PK
        uuid niche_id FK
        text status
        int target_count
        jsonb params
        timestamptz created_at
    }
    jobs {
        uuid id PK
        uuid run_id FK
        text tool
        jsonb command_spec
        text status
        int attempts
        timestamptz queued_at
        timestamptz started_at
        timestamptz finished_at
        text last_error
    }
    designs {
        uuid id PK
        uuid run_id FK
        text base_prompt
        text s3_prefix_raw
    }
    design_variants {
        uuid id PK
        uuid design_id FK
        text kind
        text s3_key
        text sha256
    }
    curations {
        uuid id PK
        uuid variant_id FK
        text decision
        text reject_reason
        uuid reviewer_id
        timestamptz decided_at
    }
    products {
        uuid id PK
        text sku
        text title
        text license_tier
        numeric price
        text s3_prefix
    }
    platform_listings {
        uuid id PK
        uuid product_id FK
        text platform
        text platform_listing_id
        text state
        numeric price
        jsonb platform_raw
    }
    sales_events {
        uuid id PK
        uuid listing_id FK
        text event_type
        numeric gross
        numeric fee
        numeric net
        timestamptz occurred_at
    }
    fee_tables {
        uuid id PK
        text platform
        jsonb rules
        timestamptz effective_from
    }
```

**Job state machine:**

```mermaid
stateDiagram-v2
    [*] --> queued : INSERT (run created)
    queued --> claimed : worker SELECT … FOR UPDATE SKIP LOCKED
    claimed --> running : spawned container
    running --> done : exit 0 + receipt ok
    running --> failed : exit !=0 or timeout
    running --> failed : validation rejected spec
    failed --> queued : attempts < max_retries (3)<br/>backoff = 2^attempts * 30s
    failed --> dead : attempts >= max_retries
    done --> [*]
    dead --> [*]
```

Transition rules:
- `queued → claimed` is atomic via `SELECT … FOR UPDATE SKIP LOCKED` — no double-claim.
- `running` jobs have a **lease**: `lock_expires_at`. If no heartbeat for 120s, reaper moves running → failed (attempts++). Guards against crashed workers.
- `failed → queued` only when `attempts < 3`. Backoff is stored in `next_attempt_at`; claim query filters `next_attempt_at <= now()`.
- `done` requires an execution receipt row (worker evidence per M2.3).

**Listing state machine:**

```mermaid
stateDiagram-v2
    [*] --> draft
    draft --> publishing : operator clicks publish (job enqueued)
    publishing --> live : adapter.publish_listing OK
    publishing --> failed : adapter error (auth/validation)
    failed --> publishing : retry
    live --> updating : price/metadata change
    updating --> live : adapter.update_listing OK
    live --> delisting : retire decision
    delisting --> delisted : adapter.update(status=inactive) OK
    live --> suspended : platform-side suspension (webhook/poll)
    suspended --> publishing : manual re-publish
    delisted --> draft : operator re-activation
```

---

