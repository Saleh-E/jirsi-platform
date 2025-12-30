-- ============================================================================
-- App Marketplace System
-- Extends apps to support marketplace publishing and installations
-- ============================================================================

-- Add marketplace fields to apps table
ALTER TABLE apps ADD COLUMN IF NOT EXISTS marketplace_id UUID;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS version VARCHAR(50) DEFAULT '1.0.0';
ALTER TABLE apps ADD COLUMN IF NOT EXISTS publisher_id UUID;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS publisher_name VARCHAR(255);
ALTER TABLE apps ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT FALSE;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS downloads INTEGER DEFAULT 0;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS rating DECIMAL(2,1);
ALTER TABLE apps ADD COLUMN IF NOT EXISTS published_at TIMESTAMPTZ;

-- App installations (tracks which tenants have installed which apps)
CREATE TABLE IF NOT EXISTS app_installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    
    -- Version tracking
    installed_version VARCHAR(50) NOT NULL,
    available_version VARCHAR(50),  -- Latest available version
    
    -- Installation status
    status VARCHAR(50) NOT NULL DEFAULT 'active'
        CHECK (status IN ('pending', 'active', 'disabled', 'uninstalled', 'update_available')),
    
    -- Configuration
    settings JSONB NOT NULL DEFAULT '{}',
    
    -- Permissions granted
    permissions JSONB NOT NULL DEFAULT '[]',  -- ['read:contacts', 'write:deals', etc.]
    
    -- Timestamps
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uninstalled_at TIMESTAMPTZ,
    
    UNIQUE(tenant_id, app_id)
);

CREATE INDEX IF NOT EXISTS idx_app_installations_tenant 
    ON app_installations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_app_installations_app 
    ON app_installations(app_id);

-- App marketplace categories
CREATE TABLE IF NOT EXISTS app_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    icon VARCHAR(10),
    description TEXT,
    sort_order INTEGER DEFAULT 0
);

-- Seed default categories
INSERT INTO app_categories (code, name, icon, description, sort_order) VALUES
    ('crm', 'CRM & Sales', '💼', 'Customer relationship management and sales tools', 1),
    ('real_estate', 'Real Estate', '🏠', 'Property management and real estate tools', 2),
    ('marketing', 'Marketing', '📣', 'Marketing automation and campaigns', 3),
    ('finance', 'Finance', '💰', 'Invoicing, payments, and accounting', 4),
    ('analytics', 'Analytics', '📊', 'Reporting and business intelligence', 5),
    ('communication', 'Communication', '💬', 'Email, SMS, and messaging tools', 6),
    ('productivity', 'Productivity', '⚡', 'Workflow automation and productivity', 7),
    ('integrations', 'Integrations', '🔗', 'Connect with external services', 8)
ON CONFLICT (code) DO NOTHING;

-- Link apps to categories (many-to-many)
CREATE TABLE IF NOT EXISTS app_category_links (
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES app_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (app_id, category_id)
);

-- App reviews/ratings
CREATE TABLE IF NOT EXISTS app_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    title VARCHAR(255),
    body TEXT,
    
    is_featured BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(app_id, tenant_id)  -- One review per tenant per app
);

CREATE INDEX IF NOT EXISTS idx_app_reviews_app 
    ON app_reviews(app_id);

-- Plugin actions (for marketplace scripts/plugins)
CREATE TABLE IF NOT EXISTS plugin_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    
    -- Plugin identity
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(10),
    
    -- Plugin type
    plugin_type VARCHAR(50) NOT NULL 
        CHECK (plugin_type IN ('workflow_node', 'trigger', 'webhook', 'ui_extension', 'background_job')),
    
    -- WASM module (if applicable)
    wasm_module_url TEXT,
    wasm_function_name VARCHAR(100),
    
    -- Input/Output schema
    input_schema JSONB,
    output_schema JSONB,
    config_schema JSONB,
    
    -- Permissions required
    required_permissions JSONB DEFAULT '[]',
    
    -- Resource limits
    timeout_ms INTEGER DEFAULT 5000,
    memory_limit_mb INTEGER DEFAULT 64,
    
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(app_id, code)
);

CREATE INDEX IF NOT EXISTS idx_plugin_actions_app 
    ON plugin_actions(app_id, is_active);

-- Plugin executions (audit log)
CREATE TABLE IF NOT EXISTS plugin_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES plugin_actions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Execution context
    workflow_execution_id UUID,
    triggered_by_user_id UUID,
    
    -- Input/Output
    input_data JSONB,
    output_data JSONB,
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'running'
        CHECK (status IN ('running', 'completed', 'failed', 'timeout')),
    error_message TEXT,
    
    -- Performance
    execution_time_ms INTEGER,
    memory_used_mb DECIMAL(10,2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_plugin_executions_plugin 
    ON plugin_executions(plugin_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_plugin_executions_tenant 
    ON plugin_executions(tenant_id, created_at DESC);

-- ============================================================================
-- CRM App Seeder
-- ============================================================================

INSERT INTO apps (id, code, name, icon, description, is_active)
VALUES (
    '11111111-1111-1111-1111-111111111111',
    'crm',
    'Jirsi CRM',
    '💼',
    'Complete customer relationship management with contacts, deals, and pipeline management.',
    TRUE
)
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description;

-- Update CRM app with marketplace fields
UPDATE apps SET
    version = '2.0.0',
    publisher_name = 'Jirsi',
    is_public = TRUE,
    downloads = 0,
    rating = 5.0,
    published_at = NOW()
WHERE code = 'crm';

-- Link CRM to categories
INSERT INTO app_category_links (app_id, category_id)
SELECT 
    (SELECT id FROM apps WHERE code = 'crm'),
    (SELECT id FROM app_categories WHERE code = 'crm')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Real Estate App Seeder
-- ============================================================================

INSERT INTO apps (id, code, name, icon, description, is_active)
VALUES (
    '22222222-2222-2222-2222-222222222222',
    'real_estate',
    'Jirsi Real Estate',
    '🏠',
    'Property management, listings, viewings, and contract lifecycle management.',
    TRUE
)
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description;

-- Update Real Estate app with marketplace fields
UPDATE apps SET
    version = '2.0.0',
    publisher_name = 'Jirsi',
    is_public = TRUE,
    downloads = 0,
    rating = 5.0,
    published_at = NOW()
WHERE code = 'real_estate';

-- Link Real Estate to categories
INSERT INTO app_category_links (app_id, category_id)
SELECT 
    (SELECT id FROM apps WHERE code = 'real_estate'),
    (SELECT id FROM app_categories WHERE code = 'real_estate')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Triggers
-- ============================================================================

CREATE OR REPLACE FUNCTION update_app_installation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_app_installations_updated
    BEFORE UPDATE ON app_installations
    FOR EACH ROW
    EXECUTE FUNCTION update_app_installation_timestamp();

-- Update app download count on installation
CREATE OR REPLACE FUNCTION increment_app_downloads()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE apps SET downloads = downloads + 1 WHERE id = NEW.app_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_app_installation_increment_downloads
    AFTER INSERT ON app_installations
    FOR EACH ROW
    EXECUTE FUNCTION increment_app_downloads();

-- Update app rating on new review
CREATE OR REPLACE FUNCTION update_app_rating()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE apps SET rating = (
        SELECT AVG(rating)::DECIMAL(2,1) 
        FROM app_reviews 
        WHERE app_id = NEW.app_id
    ) WHERE id = NEW.app_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_app_review_update_rating
    AFTER INSERT OR UPDATE ON app_reviews
    FOR EACH ROW
    EXECUTE FUNCTION update_app_rating();
