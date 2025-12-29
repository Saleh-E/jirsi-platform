-- Enable Row-Level Security on all multi-tenant tables
-- This migration ensures complete tenant isolation across all entity types

-- ============================================
-- HELPER FUNCTION
-- ============================================

-- Safe current tenant function that doesn't throw errors
CREATE OR REPLACE FUNCTION get_current_tenant() RETURNS uuid AS $$
BEGIN
    RETURN NULLIF(current_setting('app.current_tenant', true), '')::uuid;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================
-- ENTITY_RECORDS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'entity_records' AND policyname = 'tenant_isolation_entity_records') THEN
        ALTER TABLE entity_records ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_entity_records ON entity_records
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- CONTACTS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'contacts' AND policyname = 'tenant_isolation_contacts') THEN
        ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_contacts ON contacts
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- COMPANIES TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'companies' AND policyname = 'tenant_isolation_companies') THEN
        ALTER TABLE companies ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_companies ON companies
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- DEALS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'deals' AND policyname = 'tenant_isolation_deals') THEN
        ALTER TABLE deals ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_deals ON deals
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- TASKS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'tasks' AND policyname = 'tenant_isolation_tasks') THEN
        ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_tasks ON tasks
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- PROPERTIES TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'properties') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'properties' AND policyname = 'tenant_isolation_properties') THEN
            ALTER TABLE properties ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_properties ON properties
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- LISTINGS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'listings') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'listings' AND policyname = 'tenant_isolation_listings') THEN
            ALTER TABLE listings ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_listings ON listings
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- CONTRACTS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'contracts') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'contracts' AND policyname = 'tenant_isolation_contracts') THEN
            ALTER TABLE contracts ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_contracts ON contracts
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- VIEWINGS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'viewings') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'viewings' AND policyname = 'tenant_isolation_viewings') THEN
            ALTER TABLE viewings ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_viewings ON viewings
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- OFFERS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'offers') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'offers' AND policyname = 'tenant_isolation_offers') THEN
            ALTER TABLE offers ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_offers ON offers
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- WORKFLOWS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'workflows') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'workflows' AND policyname = 'tenant_isolation_workflows') THEN
            ALTER TABLE workflows ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_workflows ON workflows
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- WORKFLOW_DEFS TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'workflow_defs') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'workflow_defs' AND policyname = 'tenant_isolation_workflow_defs') THEN
            ALTER TABLE workflow_defs ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_workflow_defs ON workflow_defs
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- ACTIVITY_LOG TABLE (if exists)
-- ============================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'activity_log') THEN
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'activity_log' AND policyname = 'tenant_isolation_activity_log') THEN
            ALTER TABLE activity_log ENABLE ROW LEVEL SECURITY;
            CREATE POLICY tenant_isolation_activity_log ON activity_log
                FOR ALL
                USING (tenant_id = get_current_tenant())
                WITH CHECK (tenant_id = get_current_tenant());
        END IF;
    END IF;
END $$;

-- ============================================
-- INTERACTIONS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'interactions' AND policyname = 'tenant_isolation_interactions') THEN
        ALTER TABLE interactions ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_interactions ON interactions
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- ASSOCIATIONS TABLE
-- ============================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE tablename = 'associations' AND policyname = 'tenant_isolation_associations') THEN
        ALTER TABLE associations ENABLE ROW LEVEL SECURITY;
        CREATE POLICY tenant_isolation_associations ON associations
            FOR ALL
            USING (tenant_id = get_current_tenant())
            WITH CHECK (tenant_id = get_current_tenant());
    END IF;
END $$;

-- ============================================
-- NOTE: Using get_current_tenant() function
-- makes it return NULL instead of error if setting not found.
-- This is safer for migrations/admin operations.
-- ============================================
