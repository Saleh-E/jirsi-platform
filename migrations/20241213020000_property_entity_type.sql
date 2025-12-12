-- Phase 1B Task P1B-02: Property EntityType
-- Registers the Property entity in entity_types table

INSERT INTO entity_types (
    id, 
    tenant_id, 
    app_id, 
    name, 
    label, 
    label_plural, 
    icon
)
VALUES (
    'e0000000-0000-0000-0000-000000000010',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'property',
    'Property',
    'Properties',
    'home'
)
ON CONFLICT (tenant_id, name) DO NOTHING;


