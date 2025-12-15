-- Integrations table for storing provider configurations
-- API credentials are encrypted at rest

CREATE TABLE IF NOT EXISTS integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    -- Encrypted credentials (AES-256-GCM)
    credentials_encrypted BYTEA,
    -- Webhook secret for signature validation
    webhook_secret VARCHAR(128) NOT NULL,
    -- Generated webhook URL
    webhook_url VARCHAR(500),
    -- Tracking
    last_webhook_at TIMESTAMPTZ,
    webhook_success_count INTEGER DEFAULT 0,
    webhook_error_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Each tenant can only have one config per provider
    CONSTRAINT integrations_tenant_provider_unique UNIQUE (tenant_id, provider)
);

-- Index for quick lookup
CREATE INDEX IF NOT EXISTS idx_integrations_tenant ON integrations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_integrations_provider ON integrations(provider);

-- Webhook logs for debugging
CREATE TABLE IF NOT EXISTS webhook_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    -- Request details
    request_headers JSONB,
    request_body TEXT,
    -- Response
    response_status INTEGER,
    response_body TEXT,
    -- Processing
    events_emitted INTEGER DEFAULT 0,
    error_message TEXT,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for recent logs
CREATE INDEX IF NOT EXISTS idx_webhook_logs_tenant_time 
    ON webhook_logs(tenant_id, processed_at DESC);

-- Cleanup old logs (keep 30 days)
CREATE INDEX IF NOT EXISTS idx_webhook_logs_cleanup 
    ON webhook_logs(processed_at) 
    WHERE processed_at < NOW() - INTERVAL '30 days';
