-- Finalize Deal pipeline metadata
-- Add stage options and default kanban view for deals

DO $$
DECLARE
    sys_tenant_id UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
    deal_entity_id UUID := 'e0000000-0000-0000-0000-000000000003';
    deal_stage_field_id UUID;
BEGIN
    -- 1. Ensure Deal stage options
    UPDATE field_defs SET options = '[
        {"value": "prospecting", "label": "Prospecting", "color": "#94a3b8"},
        {"value": "qualification", "label": "Qualification", "color": "#6366f1"},
        {"value": "proposal", "label": "Proposal", "color": "#f59e0b"},
        {"value": "negotiation", "label": "Negotiation", "color": "#ec4899"},
        {"value": "closed_won", "label": "Closed Won", "color": "#22c55e"},
        {"value": "closed_lost", "label": "Closed Lost", "color": "#ef4444"}
    ]'::jsonb 
    WHERE name = 'stage' AND entity_type_id = deal_entity_id;

    -- 2. Create Default Kanban View for Deals
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, settings, columns)
    VALUES (
        gen_random_uuid(),
        sys_tenant_id,
        deal_entity_id,
        'default_kanban',
        'Sales Pipeline',
        'kanban',
        true,
        '{"group_by": "stage"}'::jsonb,
        '[
            {"field": "name", "visible": true},
            {"field": "amount", "visible": true},
            {"field": "expected_close_date", "visible": true}
        ]'::jsonb
    ) ON CONFLICT (tenant_id, entity_type_id, name) DO UPDATE 
    SET view_type = 'kanban', is_default = true, settings = '{"group_by": "stage"}'::jsonb;

END $$;
