-- Enable Row-Level Security on all multi-tenant tables
-- This migration ensures complete tenant isolation across all entity types

-- ============================================
-- PROPERTIES TABLE
-- ============================================
ALTER TABLE properties ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_properties ON properties
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- LISTINGS TABLE
-- ============================================
ALTER TABLE listings ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_listings ON listings
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- CONTRACTS TABLE
-- ============================================
ALTER TABLE contracts ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_contracts ON contracts
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- VIEWINGS TABLE
-- ============================================
ALTER TABLE viewings ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_viewings ON viewings
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- OFFERS TABLE
-- ============================================
ALTER TABLE offers ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_offers ON offers
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- WORKFLOWS TABLE
-- ============================================
ALTER TABLE workflows ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_workflows ON workflows
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- WORKFLOW_DEFS TABLE
-- ============================================
ALTER TABLE workflow_defs ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_workflow_defs ON workflow_defs
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- TASKS TABLE
-- ============================================
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_tasks ON tasks
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- CONTACTS TABLE
-- ============================================
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_contacts ON contacts
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- COMPANIES TABLE
-- ============================================
ALTER TABLE companies ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_companies ON companies
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- DEALS TABLE
-- ============================================
ALTER TABLE deals ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_deals ON deals
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- ACTIVITY_LOG TABLE
-- ============================================
ALTER TABLE activity_log ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_activity_log ON activity_log
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- ============================================
-- NOTE: Using current_setting with 'true' as second arg
-- makes it return NULL instead of error if setting not found.
-- This is safer for migrations/admin operations.
-- ============================================
