-- Phase 2 Task P2-A05: Role-Based Associations
-- Extends association system with role tracking and metadata

-- ============================================================================
-- CREATE RECORD ASSOCIATIONS TABLE
-- Links records with roles and metadata
-- ============================================================================

CREATE TABLE IF NOT EXISTS record_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Association definition reference
    association_def_id UUID NOT NULL REFERENCES association_defs(id),
    
    -- Source and target records
    source_entity_type VARCHAR(100) NOT NULL,
    source_record_id UUID NOT NULL,
    target_entity_type VARCHAR(100) NOT NULL,
    target_record_id UUID NOT NULL,
    
    -- Role-based linking
    role VARCHAR(100), -- e.g., 'owner', 'agent', 'buyer', 'seller', 'tenant', 'landlord'
    is_primary BOOLEAN DEFAULT false,
    
    -- Metadata
    start_date DATE,
    end_date DATE,
    notes TEXT,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    -- Constraints
    UNIQUE(source_record_id, target_record_id, role)
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_record_associations_tenant ON record_associations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_record_associations_source ON record_associations(source_entity_type, source_record_id);
CREATE INDEX IF NOT EXISTS idx_record_associations_target ON record_associations(target_entity_type, target_record_id);
CREATE INDEX IF NOT EXISTS idx_record_associations_role ON record_associations(role);
CREATE INDEX IF NOT EXISTS idx_record_associations_primary ON record_associations(is_primary) WHERE is_primary = true;

-- ============================================================================
-- SEED ROLE DEFINITIONS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete)
VALUES
-- Property ↔ Contact (roles)
('adef0008-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_owner', 'property', 'contact', 'Owner', 'Owned Properties', 'many_to_one', 'owner', 'owner', true, false),
('adef0008-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_developer', 'property', 'company', 'Developer', 'Developed Properties', 'many_to_one', 'developer', 'developer', false, false),
('adef0008-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_pm', 'property', 'contact', 'Property Manager', 'Managed Properties', 'many_to_one', 'manager', 'property_manager', false, false),

-- Deal roles
('adef0008-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'deal_lead', 'deal', 'contact', 'Lead Contact', 'Deals', 'many_to_one', 'lead', 'deal_lead', true, false),
('adef0008-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'deal_decision_maker', 'deal', 'contact', 'Decision Maker', 'Deals', 'many_to_one', 'decision_maker', 'decision_maker', false, false),
('adef0008-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'deal_influencer', 'deal', 'contact', 'Influencer', 'Deals', 'many_to_one', 'influencer', 'influencer', false, false),

-- Contact ↔ Company roles
('adef0008-0000-0000-0000-000000000020', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contact_employer', 'contact', 'company', 'Employer', 'Employees', 'many_to_one', 'employee', 'employer', true, false),
('adef0008-0000-0000-0000-000000000021', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contact_sponsor', 'contact', 'company', 'Sponsor', 'Sponsored Contacts', 'many_to_one', 'sponsored', 'sponsor', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- ASSOCIATIONS API VIEW
-- For easy querying of associations with resolved names
-- ============================================================================

CREATE OR REPLACE VIEW v_record_associations AS
SELECT 
    ra.id,
    ra.tenant_id,
    ra.association_def_id,
    ad.name as association_name,
    ad.label_source,
    ad.label_target,
    ra.source_entity_type,
    ra.source_record_id,
    ra.target_entity_type,
    ra.target_record_id,
    ra.role,
    ra.is_primary,
    ra.start_date,
    ra.end_date,
    ra.created_at
FROM record_associations ra
JOIN association_defs ad ON ra.association_def_id = ad.id;
