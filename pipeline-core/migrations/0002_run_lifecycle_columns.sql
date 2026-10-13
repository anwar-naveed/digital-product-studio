-- Digital Product Studio - Schema alignment migration
-- Adds run lifecycle columns required by pipeline-core queue/api code.

ALTER TABLE production_runs
    ADD COLUMN IF NOT EXISTS started_at timestamptz,
    ADD COLUMN IF NOT EXISTS completed_at timestamptz,
    ADD COLUMN IF NOT EXISTS last_error text;

CREATE INDEX IF NOT EXISTS idx_production_runs_started_at ON production_runs(started_at);

-- fee_tables: one active fee table per platform (upsert target)
CREATE UNIQUE INDEX IF NOT EXISTS uq_fee_tables_platform ON fee_tables(platform);
