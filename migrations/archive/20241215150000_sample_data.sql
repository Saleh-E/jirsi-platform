-- Phase 3 Task P3-15: Sample Data Seeding
-- Seed sample activity log entries to test timeline

-- ============================================================================
-- SAMPLE ACTIVITY LOG ENTRIES
-- ============================================================================

INSERT INTO activity_log (id, tenant_id, activity_type, title, description, entity_type, entity_id, occurred_at)
SELECT
    gen_random_uuid(),
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    types.activity_type,
    types.activity_title,
    types.activity_desc,
    'property',
    p.id,
    NOW() - (random() * 30)::int * INTERVAL '1 day'
FROM (SELECT id FROM properties WHERE tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e' LIMIT 3) p
CROSS JOIN (VALUES 
    ('note', 'Added property details', 'Updated the property description and photos'),
    ('call', 'Client inquiry call', 'Discussed pricing and availability'),
    ('status_change', 'Status changed to Active', 'Property is now live on the market')
) AS types(activity_type, activity_title, activity_desc)
ON CONFLICT DO NOTHING;

-- Note: Viewings, Offers, and Contracts sample data should be created via the UI
-- to ensure all foreign key constraints (agent_id, contact_id) are satisfied
