-- Finalize Deal pipeline metadata
-- Add stage options and default kanban view for deals

DO $$
DECLARE
    sys_tenant_id UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
    deal_entity_id UUID := 'e0000000-0000-0000-0000-000000000003';
    existing_view_id UUID;
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

    -- 2. Check if kanban view already exists
    SELECT id INTO existing_view_id 
    FROM view_defs 
    WHERE tenant_id = sys_tenant_id 
      AND entity_type_id = deal_entity_id 
      AND view_type = 'kanban'
    LIMIT 1;

    -- 3. Create Default Kanban View for Deals if not exists
    IF existing_view_id IS NULL THEN
        INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, settings, columns)
        VALUES (
            gen_random_uuid(),
            sys_tenant_id,
            deal_entity_id,
            'deals_kanban',
            'Sales Pipeline',
            'kanban',
            false,
            true,
            '{"group_by_field": "stage"}'::jsonb,
            '[
                {"field": "name", "visible": true},
                {"field": "amount", "visible": true},
                {"field": "expected_close_date", "visible": true}
            ]'::jsonb
        );
    END IF;

END $$;
