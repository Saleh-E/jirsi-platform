-- Migrate Deals and Tasks to entity_records

-- Helper to get entity_type_id
DO $$
DECLARE
    crm_app_id UUID;
    deal_type_id UUID;
    task_type_id UUID;
BEGIN
    SELECT id INTO crm_app_id FROM app_defs WHERE id = 'crm';

    -- Ensure entity types exist (idempotent-ish check/insert)
    -- Ideally these are seeded, but for migration safety we ensure we can find them.
    -- Assuming they were created in Phase 1 or we use the ones from seed.
    SELECT id INTO deal_type_id FROM entity_types WHERE name = 'deal' LIMIT 1;
    SELECT id INTO task_type_id FROM entity_types WHERE name = 'task' LIMIT 1;

    -- Migrate Deals
    INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at, deleted_at)
    SELECT 
        id,
        tenant_id,
        deal_type_id,
        jsonb_build_object(
            'name', name,
            'amount', amount,
            'stage', stage,
            'expected_close_date', expected_close_date,
            'probability', probability,
            'pipeline_id', pipeline_id
        ),
        created_at,
        updated_at,
        deleted_at
    FROM deals;

    -- Migrate Tasks
    INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at, deleted_at)
    SELECT 
        id,
        tenant_id,
        task_type_id,
        jsonb_build_object(
            'title', title,
            'description', description,
            'status', status,
            'priority', priority,
            'due_date', due_date,
            'assignee_id', assignee_id,
            'completed_at', completed_at,
            'linked_entity_type', linked_entity_type,
            'linked_entity_id', linked_entity_id
        ),
        created_at,
        updated_at,
        NULL -- No deleted_at in source tasks table
    FROM tasks;

END $$;
