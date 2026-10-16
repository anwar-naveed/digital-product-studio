-- 0005: platform credential storage (encrypted at rest, AES-256-GCM by the API).
-- G3: secrets are WRITE-ONLY through the API; GET responses only ever expose
-- masked field names.

CREATE TABLE IF NOT EXISTS platform_credentials (
    platform    text NOT NULL CHECK (platform IN ('etsy', 'lemon_squeezy', 'printify', 'gelato')),
    field       text NOT NULL CHECK (field IN (
                    -- Etsy Open API v3 (OAuth2)
                    'keystring', 'shared_secret', 'access_token', 'refresh_token', 'expires_at', 'shop_id',
                    -- Lemon Squeezy (Bearer API key)
                    'api_key', 'store_id',
                    -- Etsy webhooks / LS webhooks
                    'webhook_secret'
                )),
    encrypted_value  bytea NOT NULL,
    nonce            bytea NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (platform, field)
);

CREATE INDEX IF NOT EXISTS idx_platform_credentials_platform ON platform_credentials(platform);
