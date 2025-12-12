-- Phase 1B Task P1B-01: Real Estate App Definition
-- Adds Real Estate app to the platform's app registry

INSERT INTO app_defs (id, tenant_id, name, label, icon, description, sort_order, is_enabled)
VALUES (
    'realestate',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'Real Estate',
    'building',
    'Property management, listings, and sales',
    2,
    true
)
ON CONFLICT (id) DO NOTHING;
