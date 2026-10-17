-- 0006: draft marketplace listing metadata on catalog products.
-- Stores what WILL be published per platform (Etsy title/tags/description,
-- LemonSqueezy name/description/license) so the UI can render a publish
-- preview before any platform credentials exist.
ALTER TABLE products ADD COLUMN IF NOT EXISTS listing_metadata JSONB;
