-- Phase 1B Task P1B-07: Property AssociationDefs
-- Defines relationships between Property and other entities

-- Property <-> Contact associations (Owner, Buyer, Tenant, Agent, Interested)
INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
-- Owner relationship
('adef0001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 
 'property_owner', 'property', 'contact', 'Owner', 'Owned Properties', 'many_to_one', true, false),

-- Buyer relationship (for sales)
('adef0001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 
 'property_buyer', 'property', 'contact', 'Buyer', 'Purchased Properties', 'many_to_one', false, false),

-- Tenant relationship (for rentals)
('adef0001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 
 'property_tenant', 'property', 'contact', 'Tenant', 'Rented Properties', 'many_to_one', false, false),

-- Listing Agent relationship
('adef0001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 
 'property_agent', 'property', 'contact', 'Listing Agent', 'Assigned Properties', 'many_to_one', false, false),

-- Interested Contacts (many-to-many)
('adef0001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 
 'property_interested', 'property', 'contact', 'Interested Contacts', 'Interested Properties', 'many_to_many', false, false)

ON CONFLICT (id) DO NOTHING;

-- Property <-> Company associations (Developer)
INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity,
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
('adef0002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_developer', 'property', 'company', 'Developer', 'Developed Properties', 'many_to_one', false, false)
ON CONFLICT (id) DO NOTHING;

-- Property <-> Deal associations
INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity,
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
('adef0003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'property_deals', 'property', 'deal', 'Related Deals', 'Property', 'one_to_many', false, false)
ON CONFLICT (id) DO NOTHING;
