-- Complete fix for missing data

-- 1. CREATE DEFAULT PIPELINE for Deals
INSERT INTO pipelines (id, tenant_id, name, is_default, stages)
VALUES (
    'p0000001-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'Sales Pipeline',
    true,
    '[
        {"id": "lead", "name": "Lead", "probability": 10},
        {"id": "qualified", "name": "Qualified", "probability": 25},
        {"id": "proposal", "name": "Proposal", "probability": 50},
        {"id": "negotiation", "name": "Negotiation", "probability": 75},
        {"id": "closed_won", "name": "Closed Won", "probability": 100},
        {"id": "closed_lost", "name": "Closed Lost", "probability": 0}
    ]'::jsonb
) ON CONFLICT DO NOTHING;

-- 2. CREATE DEFAULT USER for Tasks (admin user)
INSERT INTO users (id, tenant_id, email, name, password_hash, role, status)
VALUES (
    'u0000001-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'admin@demo.com',
    'Admin User',
    '$argon2id$v=19$m=19456,t=2,p=1$xxxxxxxxx',
    'admin',
    'active'
) ON CONFLICT DO NOTHING;

-- 3. FIX LOOKUP FIELDS - Add ui_hints to specify target entity

-- Viewing: property_id should lookup properties
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb 
WHERE id = 'f2000001-0000-0000-0000-000000000001';

-- Viewing: contact_id should lookup contacts
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f2000001-0000-0000-0000-000000000002';

-- Viewing: agent_id should lookup contacts (agents are contacts)
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f2000001-0000-0000-0000-000000000003';

-- Offer: property_id should lookup properties
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb 
WHERE id = 'f3000001-0000-0000-0000-000000000001';

-- Offer: contact_id should lookup contacts
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f3000001-0000-0000-0000-000000000002';

-- Offer: deal_id should lookup deals
UPDATE field_defs SET ui_hints = '{"lookup_entity": "deal"}'::jsonb 
WHERE id = 'f3000001-0000-0000-0000-000000000003';

-- Property: owner_id, agent_id, developer_id should lookup contacts
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f1000001-0000-0000-0000-000000000023';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f1000001-0000-0000-0000-000000000024';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE id = 'f1000001-0000-0000-0000-000000000025';

-- Fix Listing lookup fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb 
WHERE name = 'property_id' AND entity_type_id = (SELECT id FROM entity_types WHERE name = 'listing' AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e');

-- Fix Contract lookup fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb 
WHERE name = 'property_id' AND entity_type_id = (SELECT id FROM entity_types WHERE name = 'contract' AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e');

UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name IN ('buyer_id', 'seller_id') AND entity_type_id = (SELECT id FROM entity_types WHERE name = 'contract' AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e');

-- Fix all property_id lookups
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb 
WHERE name = 'property_id' AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e' AND ui_hints IS NULL;

-- Fix all contact_id lookups  
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'contact_id' AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e' AND ui_hints IS NULL;

-- 4. Update Deal field defs to include pipeline_id (the field likely exists but needs proper config)
-- First check if deal entity has proper fields with options
UPDATE field_defs SET options = '[
    {"value": "lead", "label": "Lead"},
    {"value": "qualified", "label": "Qualified"},
    {"value": "proposal", "label": "Proposal"},
    {"value": "negotiation", "label": "Negotiation"},
    {"value": "closed_won", "label": "Closed Won"},
    {"value": "closed_lost", "label": "Closed Lost"}
]'::jsonb WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000003' AND name = 'stage';

-- Verify counts
SELECT 'Pipelines:' as item, COUNT(*) as count FROM pipelines WHERE tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e'
UNION ALL
SELECT 'Users:', COUNT(*) FROM users WHERE tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e'
UNION ALL
SELECT 'Lookup fields fixed:', COUNT(*) FROM field_defs WHERE ui_hints IS NOT NULL AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
