-- Phase 3 Task P3-05: Role-Based Associations
-- Define association roles between Properties, Contacts, Companies, and Deals

-- ============================================================================
-- ROLE-BASED ASSOCIATION DEFINITIONS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete)
VALUES
-- Property → Contact roles
('ae000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_owner', 'property', 'contact', 'Owner', 'Owned Properties', 'many_to_one', 'owner', 'owner', true, false),
('ae000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_buyer', 'property', 'contact', 'Buyer', 'Properties Bought', 'many_to_one', 'buyer', 'buyer', false, false),
('ae000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_tenant', 'property', 'contact', 'Tenant', 'Rented Properties', 'many_to_one', 'tenant', 'tenant', false, false),
('ae000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_agent', 'property', 'contact', 'Agent', 'Managed Properties', 'many_to_one', 'agent', 'agent', false, false),
('ae000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_interested', 'property', 'contact', 'Interested Party', 'Properties Interested', 'many_to_many', 'interested', 'interested', false, false),

-- Property → Company roles
('ae000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_developer', 'property', 'company', 'Developer', 'Developed Properties', 'many_to_one', 'developer', 'developer', false, false),

-- Viewing → Contact roles
('ae000001-0000-0000-0000-000000000020', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_attendee', 'viewing', 'contact', 'Attendee', 'Viewings Attended', 'many_to_one', 'attendee', 'viewing_attendee', true, false),

-- Offer → Contact roles  
('ae000001-0000-0000-0000-000000000030', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_offerer', 'offer', 'contact', 'Offerer', 'Offers Made', 'many_to_one', 'offerer', 'offerer', true, false),

-- Contract → Contact roles
('ae000001-0000-0000-0000-000000000040', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_buyer', 'contract', 'contact', 'Buyer', 'Contracts (Buyer)', 'many_to_one', 'buyer', 'contract_buyer', true, false),
('ae000001-0000-0000-0000-000000000041', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_seller', 'contract', 'contact', 'Seller', 'Contracts (Seller)', 'many_to_one', 'seller', 'contract_seller', false, false),

-- Property → Deal
('ae000001-0000-0000-0000-000000000050', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_deal', 'property', 'deal', 'Deal', 'Properties', 'many_to_one', 'property', 'deal', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- SAMPLE RECORD ASSOCIATIONS
-- ============================================================================

-- (Will be populated when records are created with associations)

-- ============================================================================
-- UPDATE ASSOCIATION VIEW
-- ============================================================================

CREATE OR REPLACE VIEW v_entity_associations AS
SELECT 
    ad.id as association_def_id,
    ad.tenant_id,
    ad.name as association_name,
    ad.source_entity,
    ad.target_entity,
    ad.label_source,
    ad.label_target,
    ad.source_role,
    ad.target_role,
    ad.cardinality
FROM association_defs ad
WHERE ad.source_role IS NOT NULL OR ad.target_role IS NOT NULL;

-- ============================================================================
-- INDEXES FOR ROLE QUERIES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_association_defs_source_role ON association_defs(source_role) WHERE source_role IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_association_defs_target_role ON association_defs(target_role) WHERE target_role IS NOT NULL;
