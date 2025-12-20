-- Restore missing CRM Metadata (Contact, Company, Deal, Task)
-- This migration inserts the entity type definitions that were missing after the reset.

DO $$
DECLARE
    -- System Tenant (Legacy ID)
    sys_tenant_id UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
BEGIN

    -- Contact (e...01)
    INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
    VALUES (
        'e0000000-0000-0000-0000-000000000001',
        sys_tenant_id,
        'crm',
        'contact',
        'Contact',
        'Contacts',
        'users'
    ) ON CONFLICT (tenant_id, name) DO NOTHING;

    -- Company (e...02)
    INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
    VALUES (
        'e0000000-0000-0000-0000-000000000002',
        sys_tenant_id,
        'crm',
        'company',
        'Company',
        'Companies',
        'building'
    ) ON CONFLICT (tenant_id, name) DO NOTHING;

    -- Deal (e...03)
    INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
    VALUES (
        'e0000000-0000-0000-0000-000000000003',
        sys_tenant_id,
        'crm',
        'deal',
        'Deal',
        'Deals',
        'dollar-sign'
    ) ON CONFLICT (tenant_id, name) DO NOTHING;

    -- Task (e...04)
    INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
    VALUES (
        'e0000000-0000-0000-0000-000000000004',
        sys_tenant_id,
        'crm',
        'task',
        'Task',
        'Tasks',
        'check-square'
    ) ON CONFLICT (tenant_id, name) DO NOTHING;

    -- ==================================================================================
    -- FIELD DEFINITIONS
    -- ==================================================================================

    -- Contact Fields
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
    VALUES
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'first_name', 'First Name', 'text', true, true, 1),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'last_name', 'Last Name', 'text', true, true, 2),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'email', 'Email', 'email', false, true, 3),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'phone', 'Phone', 'tel', false, true, 4),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'company_id', 'Company', 'lookup', false, true, 5),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000001', 'lifecycle_stage', 'Stage', 'select', true, true, 6)
    ON CONFLICT (entity_type_id, name) DO NOTHING;

    -- Company Fields
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
    VALUES
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000002', 'name', 'Company Name', 'text', true, true, 1),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000002', 'domain', 'Domain', 'url', false, true, 2),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000002', 'phone', 'Phone', 'tel', false, true, 3),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000002', 'city', 'City', 'text', false, true, 4)
    ON CONFLICT (entity_type_id, name) DO NOTHING;

    -- Deal Fields
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
    VALUES
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000003', 'name', 'Deal Name', 'text', true, true, 1),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000003', 'amount', 'Amount', 'currency', false, true, 2),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000003', 'stage', 'Stage', 'select', true, true, 3),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000003', 'pipeline_id', 'Pipeline', 'lookup', true, false, 4),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000003', 'expected_close_date', 'Close Date', 'date', false, true, 5)
    ON CONFLICT (entity_type_id, name) DO NOTHING;

    -- Task Fields
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
    VALUES
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000004', 'title', 'Title', 'text', true, true, 1),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000004', 'status', 'Status', 'select', true, true, 2),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000004', 'priority', 'Priority', 'select', true, true, 3),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000004', 'due_date', 'Due Date', 'datetime', false, true, 4),
    (gen_random_uuid(), sys_tenant_id, 'e0000000-0000-0000-0000-000000000004', 'assignee_id', 'Assignee', 'lookup', false, true, 5)
    ON CONFLICT (entity_type_id, name) DO NOTHING;

    -- Options for Status/Priority/Stage
    UPDATE field_defs SET options = '[
        {"value": "lead", "label": "Lead"},
        {"value": "new", "label": "New"},
        {"value": "customer", "label": "Customer"}
    ]'::jsonb WHERE name = 'lifecycle_stage' AND entity_type_id = 'e0000000-0000-0000-0000-000000000001';

    UPDATE field_defs SET options = '[
        {"value": "open", "label": "Open"},
        {"value": "in_progress", "label": "In Progress"},
        {"value": "completed", "label": "Completed"}
    ]'::jsonb WHERE name = 'status' AND entity_type_id = 'e0000000-0000-0000-0000-000000000004';

END $$;
