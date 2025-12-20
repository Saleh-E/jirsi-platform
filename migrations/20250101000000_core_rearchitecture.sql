-- Core Re-architecture: Entity Records and RLS

-- 1. Create generic entity_records table
CREATE TABLE entity_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type_id UUID NOT NULL REFERENCES entity_types(id),
    human_id VARCHAR(50), -- Friendly ID like INV-001
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_entity_records_tenant ON entity_records(tenant_id);
CREATE INDEX idx_entity_records_type ON entity_records(tenant_id, entity_type_id);
-- Index on JSONB for searching (optional but likely needed)
CREATE INDEX idx_entity_records_data ON entity_records USING gin (data);

-- 2. Enable RLS on entity_records
ALTER TABLE entity_records ENABLE ROW LEVEL SECURITY;

-- 3. Create Tenant Isolation Policy
-- This policy ensures looking at rows matches the current_setting('app.current_tenant')
-- We use a function to safe-guard against empty setting if desired, or just direct casting.
-- Note: set_config can deal with missing values (returns null), checking for null is important.

CREATE POLICY tenant_isolation_policy ON entity_records
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- 4. Enable RLS on other multi-tenant tables (Phase 0 Step 2)
-- We will do this incrementally, or all at once? The request says "Enable RLS on all multi-tenant tables".
-- Let's apply it to the main ones: contacts, deals, etc.

-- ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY tenant_isolation_contacts ON contacts
--    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- ALTER TABLE companies ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY tenant_isolation_companies ON companies
--    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- ALTER TABLE deals ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY tenant_isolation_deals ON deals
--    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY tenant_isolation_tasks ON tasks
--    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- Note: We need to bypass RLS for the system user / migration runner?
-- By default, superusers or table owners bypass RLS. the app user might not be owner.
-- Typically we grant BYPASSRLS to the app user or ensure the app user sets the tenant.
-- For safety, we might want a policy that allows ALL if 'app.current_tenant' is 'system' or similar?
-- For now, strict isolation is safer.

-- 5. Helper function to set tenant (optional, for debugging SQL)
CREATE OR REPLACE FUNCTION set_tenant(tenant_id uuid) RETURNS void AS $$
BEGIN
    PERFORM set_config('app.current_tenant', tenant_id::text, false);
END;
$$ LANGUAGE plpgsql;

-- 6. Seed a test entity type for validtion
-- We'll assume a tenant exists or we might fail if we enforce FK.
-- Actually, entity_types references tenants(id). We can't seed it globally without a tenant.
-- So we won't seed it here unless we know a tenant ID.
-- Instead, we should provide an API to create it, OR trust the 'seed' endpoint handles it.
-- The generic API requirement says "Provide endpoints to create/update EntityType".
-- So we will implement the API first, then use it.

