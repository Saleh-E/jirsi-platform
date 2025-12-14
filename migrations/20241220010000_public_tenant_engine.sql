-- Public Tenant Engine: Add is_published to properties
-- Allows controlling visibility of properties on public website

ALTER TABLE properties ADD COLUMN IF NOT EXISTS is_published BOOLEAN NOT NULL DEFAULT false;

-- Index for efficient public listing queries
CREATE INDEX IF NOT EXISTS idx_properties_published 
    ON properties(tenant_id, status, is_published) 
    WHERE is_published = true AND status = 'active';

-- Seed some properties as published for testing
UPDATE properties SET is_published = true WHERE status = 'active' AND deleted_at IS NULL;
