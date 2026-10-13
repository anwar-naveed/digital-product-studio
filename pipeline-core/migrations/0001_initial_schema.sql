-- Digital Product Studio - Initial Schema Migration
-- Creates all 12 tables per docs/11-architecture.md §2 ERD
-- PostgreSQL 16 compatible

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";  -- for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- 1. NICHES TABLE
-- ============================================================
CREATE TABLE niches (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name            text NOT NULL,
    vertical        text NOT NULL CHECK (vertical IN ('printable_wall_art', 'planner_journal_bundles', 'party_wedding_printables')),
    demand_signals  jsonb NOT NULL DEFAULT '{}'::jsonb,
    competition_score numeric NOT NULL DEFAULT 0,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_niches_vertical ON niches(vertical);
CREATE INDEX idx_niches_created_at ON niches(created_at DESC);

-- ============================================================
-- 2. PRODUCTION_RUNS TABLE
-- ============================================================
CREATE TABLE production_runs (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    niche_id      uuid NOT NULL REFERENCES niches(id) ON DELETE CASCADE,
    status        text NOT NULL CHECK (status IN ('pending', 'generating', 'curating', 'assembling', 'publishing', 'completed', 'failed', 'cancelled')),
    target_count  int NOT NULL DEFAULT 0 CHECK (target_count >= 0),
    params        jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at    timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_production_runs_niche_id ON production_runs(niche_id);
CREATE INDEX idx_production_runs_status ON production_runs(status);
CREATE INDEX idx_production_runs_created_at ON production_runs(created_at DESC);

-- ============================================================
-- 3. JOBS TABLE
-- ============================================================
CREATE TABLE jobs (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id          uuid NOT NULL REFERENCES production_runs(id) ON DELETE CASCADE,
    tool            text NOT NULL CHECK (tool IN ('cf-image-gen', 'photocraft-cli', 'vectorcraft-cli', 'webhook-processor', 'product-assembly', 'publish-listing')),
    command_spec    jsonb NOT NULL,
    status          text NOT NULL CHECK (status IN ('queued', 'claimed', 'running', 'done', 'failed', 'dead')),
    attempts        int NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    queued_at       timestamptz NOT NULL DEFAULT now(),
    started_at      timestamptz,
    finished_at     timestamptz,
    last_error      text,
    next_attempt_at timestamptz,
    lock_expires_at timestamptz
);

CREATE INDEX idx_jobs_run_id ON jobs(run_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_queued_at ON jobs(queued_at);
CREATE INDEX idx_jobs_next_attempt_at ON jobs(next_attempt_at) WHERE next_attempt_at IS NOT NULL;
CREATE INDEX idx_jobs_lock_expires_at ON jobs(lock_expires_at) WHERE lock_expires_at IS NOT NULL;

-- ============================================================
-- 4. JOB_RECEIPTS TABLE
-- ============================================================
CREATE TABLE job_receipts (
    id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id           uuid NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    command_hash     text NOT NULL,
    args_hash        text NOT NULL,
    exit_code        int NOT NULL,
    stdout_sha256    text NOT NULL,
    stderr_sha256    text NOT NULL,
    output_files     jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_job_receipts_job_id ON job_receipts(job_id);

-- ============================================================
-- 5. DESIGNS TABLE
-- ============================================================
CREATE TABLE designs (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id          uuid NOT NULL REFERENCES production_runs(id) ON DELETE CASCADE,
    base_prompt     text NOT NULL,
    s3_prefix_raw   text NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_designs_run_id ON designs(run_id);
CREATE INDEX idx_designs_created_at ON designs(created_at DESC);

-- ============================================================
-- 6. DESIGN_VARIANTS TABLE
-- ============================================================
CREATE TABLE design_variants (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    design_id  uuid NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
    kind       text NOT NULL,
    s3_key     text NOT NULL,
    sha256     text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_design_variants_design_id ON design_variants(design_id);
CREATE INDEX idx_design_variants_sha256 ON design_variants(sha256);
CREATE UNIQUE INDEX idx_design_variants_unique_sha256 ON design_variants(sha256);

-- ============================================================
-- 7. CURATIONS TABLE
-- ============================================================
CREATE TABLE curations (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    variant_id   uuid NOT NULL REFERENCES design_variants(id) ON DELETE CASCADE,
    decision     text NOT NULL CHECK (decision IN ('pending', 'approved', 'rejected')),
    reject_reason text CHECK (reject_reason IN ('style_mismatch', 'artifact', 'license', 'off_niche', 'duplicate')),
    reviewer_id  uuid NOT NULL,
    decided_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_curations_variant_id ON curations(variant_id);
CREATE INDEX idx_curations_decision ON curations(decision);
CREATE INDEX idx_curations_decided_at ON curations(decided_at DESC);

-- ============================================================
-- 8. PRODUCTS TABLE
-- ============================================================
CREATE TABLE products (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sku          text NOT NULL UNIQUE,
    title        text NOT NULL,
    license_tier text NOT NULL CHECK (license_tier IN ('personal', 'small_commercial', 'extended_commercial')),
    price        numeric NOT NULL CHECK (price >= 0),
    s3_prefix    text NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_license_tier ON products(license_tier);
CREATE INDEX idx_products_created_at ON products(created_at DESC);

-- ============================================================
-- 9. PRODUCT_ITEMS TABLE
-- ============================================================
CREATE TABLE product_items (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id      uuid NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id      uuid NOT NULL REFERENCES design_variants(id) ON DELETE CASCADE,
    position        int NOT NULL DEFAULT 0,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_items_product_id ON product_items(product_id);
CREATE INDEX idx_product_items_variant_id ON product_items(variant_id);
CREATE UNIQUE INDEX idx_product_items_unique ON product_items(product_id, variant_id);

-- ============================================================
-- 12. FEE_TABLES TABLE
-- ============================================================
CREATE TABLE fee_tables (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    platform        text NOT NULL CHECK (platform IN ('etsy', 'lemon_squeezy', 'printify', 'gelato')),
    rules           jsonb NOT NULL DEFAULT '{}'::jsonb,
    effective_from  timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_fee_tables_platform ON fee_tables(platform);
CREATE INDEX idx_fee_tables_effective_from ON fee_tables(effective_from DESC);


-- ============================================================
-- 10. PLATFORM_LISTINGS TABLE
-- ============================================================
CREATE TABLE platform_listings (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id          uuid NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    platform            text NOT NULL CHECK (platform IN ('etsy', 'lemon_squeezy', 'printify', 'gelato')),
    platform_listing_id text NOT NULL,
    state               text NOT NULL CHECK (state IN ('draft', 'publishing', 'live', 'updating', 'failed', 'delisting', 'delisted', 'suspended')),
    price               numeric NOT NULL CHECK (price >= 0),
    platform_raw        jsonb NOT NULL DEFAULT '{}'::jsonb,
    fee_table_id        uuid REFERENCES fee_tables(id),
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_platform_listings_product_id ON platform_listings(product_id);
CREATE INDEX idx_platform_listings_platform ON platform_listings(platform);
CREATE INDEX idx_platform_listings_state ON platform_listings(state);
CREATE INDEX idx_platform_listings_platform_listing_id ON platform_listings(platform, platform_listing_id);
CREATE UNIQUE INDEX idx_platform_listings_unique ON platform_listings(product_id, platform);

-- ============================================================
-- 11. SALES_EVENTS TABLE
-- ============================================================
CREATE TABLE sales_events (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id    uuid NOT NULL REFERENCES platform_listings(id) ON DELETE CASCADE,
    event_type    text NOT NULL,
    gross         numeric NOT NULL CHECK (gross >= 0),
    fee           numeric NOT NULL CHECK (fee >= 0),
    net           numeric NOT NULL,
    occurred_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_sales_events_listing_id ON sales_events(listing_id);
CREATE INDEX idx_sales_events_occurred_at ON sales_events(occurred_at DESC);
CREATE INDEX idx_sales_events_event_type ON sales_events(event_type);


-- ============================================================
-- 13. LISTING_STATES TABLE (audit/history for listing state machine)
-- ============================================================
CREATE TABLE listing_states (
    id                uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id        uuid NOT NULL REFERENCES platform_listings(id) ON DELETE CASCADE,
    from_state        text CHECK (from_state IN ('draft', 'publishing', 'live', 'updating', 'failed', 'delisting', 'delisted', 'suspended')),
    to_state          text NOT NULL CHECK (to_state IN ('draft', 'publishing', 'live', 'updating', 'failed', 'delisting', 'delisted', 'suspended')),
    transition_reason text,
    actor_id          uuid,
    created_at        timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_listing_states_listing_id ON listing_states(listing_id);
CREATE INDEX idx_listing_states_created_at ON listing_states(created_at DESC);

-- ============================================================
-- 14. PROCESSED_WEBHOOK_IDS CACHE TABLE (for idempotency per M6.2)
-- ============================================================
CREATE TABLE processed_webhook_ids (
    platform_event_id text NOT NULL,
    platform          text NOT NULL CHECK (platform IN ('etsy', 'lemon_squeezy', 'printify', 'gelato')),
    received_at       timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (platform, platform_event_id)
);

CREATE INDEX idx_processed_webhook_ids_received_at ON processed_webhook_ids(received_at);

-- ============================================================
-- HELPER FUNCTIONS AND TRIGGERS
-- ============================================================

-- Trigger function to auto-update updated_at on platform_listings
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_platform_listings_updated_at
    BEFORE UPDATE ON platform_listings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to clean up expired webhook cache entries (TTL 7 days)
CREATE OR REPLACE FUNCTION cleanup_processed_webhook_ids()
RETURNS void AS $$
BEGIN
    DELETE FROM processed_webhook_ids
    WHERE received_at < now() - interval '7 days';
END;
$$ language 'plpgsql';

-- ============================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================

COMMENT ON TABLE niches IS 'Niche verticals for product generation (R1: exactly 3 verticals)';
COMMENT ON TABLE production_runs IS 'Production runs spawned from niche selection';
COMMENT ON TABLE jobs IS 'Individual job units in the worker queue (Postgres SKIP LOCKED)';
COMMENT ON TABLE job_receipts IS 'Execution receipts from workers (M2.3 evidence)';
COMMENT ON TABLE designs IS 'Base designs generated from cf-image-gen';
COMMENT ON TABLE design_variants IS 'Variants of designs (upscaled, colorways, formats)';
COMMENT ON TABLE curations IS 'Human curation decisions (approve/reject with taxonomy)';
COMMENT ON TABLE products IS 'Assembled product bundles with pricing tiers (R15)';
COMMENT ON TABLE product_items IS 'Junction linking product bundles to design variants';
COMMENT ON TABLE platform_listings IS 'Published listings on marketplace platforms';
COMMENT ON TABLE sales_events IS 'Revenue tracking per listing with fee calculations';
COMMENT ON TABLE fee_tables IS 'Platform fee rules for margin calculation (R9)';
COMMENT ON TABLE listing_states IS 'Audit trail for listing state machine transitions';
COMMENT ON TABLE processed_webhook_ids IS 'Idempotency cache for webhook processing (M6.2, TTL 7 days)';