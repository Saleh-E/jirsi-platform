-- Complete View Definitions for All Entity Types
-- Ensures ViewSwitcher shows multiple tabs on every entity page

-- Demo tenant ID
DO $$
DECLARE
    tenant UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
    et_contact UUID;
    et_company UUID;
    et_deal UUID;
    et_task UUID;
    et_property UUID;
    et_listing UUID;
    et_viewing UUID;
    et_offer UUID;
    et_contract UUID;
BEGIN

-- Lookup entity type IDs
SELECT id INTO et_contact FROM entity_types WHERE tenant_id = tenant AND name = 'contact' LIMIT 1;
SELECT id INTO et_company FROM entity_types WHERE tenant_id = tenant AND name = 'company' LIMIT 1;
SELECT id INTO et_deal FROM entity_types WHERE tenant_id = tenant AND name = 'deal' LIMIT 1;
SELECT id INTO et_task FROM entity_types WHERE tenant_id = tenant AND name = 'task' LIMIT 1;
SELECT id INTO et_property FROM entity_types WHERE tenant_id = tenant AND name = 'property' LIMIT 1;
SELECT id INTO et_listing FROM entity_types WHERE tenant_id = tenant AND name = 'listing' LIMIT 1;
SELECT id INTO et_viewing FROM entity_types WHERE tenant_id = tenant AND name = 'viewing' LIMIT 1;
SELECT id INTO et_offer FROM entity_types WHERE tenant_id = tenant AND name = 'offer' LIMIT 1;
SELECT id INTO et_contract FROM entity_types WHERE tenant_id = tenant AND name = 'contract' LIMIT 1;

-- =====================================================
-- CONTACT VIEWS
-- =====================================================
IF et_contact IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_contact, 'contact_table', 'Table', 'table', true, 
        '["first_name", "last_name", "email", "phone", "lifecycle_stage"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_contact, 'contact_kanban', 'Pipeline', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "lifecycle_stage", "card_title_field": "first_name", "card_subtitle_field": "email"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- COMPANY VIEWS
-- =====================================================
IF et_company IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_company, 'company_table', 'Table', 'table', true, 
        '["name", "domain", "industry", "phone"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_company, 'company_kanban', 'By Industry', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "industry", "card_title_field": "name", "card_subtitle_field": "domain"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- DEAL VIEWS
-- =====================================================
IF et_deal IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_deal, 'deal_table', 'Table', 'table', true, 
        '["name", "amount", "stage", "expected_close_date"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_deal, 'deal_kanban', 'Pipeline', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "stage", "card_title_field": "name", "card_subtitle_field": "amount"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- TASK VIEWS
-- =====================================================
IF et_task IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_task, 'task_table', 'Table', 'table', true, 
        '["title", "status", "priority", "due_date"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_task, 'task_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "title", "card_subtitle_field": "due_date"}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_task, 'task_calendar', 'Calendar', 'calendar', false, 
        '[]'::jsonb, '{"date_field": "due_date", "title_field": "title", "color_field": "priority"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- PROPERTY VIEWS
-- =====================================================
IF et_property IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_property, 'property_table', 'Table', 'table', true, 
        '["reference", "title", "property_type", "status", "city", "price"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_property, 'property_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "title", "card_subtitle_field": "city"}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_property, 'property_map', 'Map', 'map', false, 
        '[]'::jsonb, '{"lat_field": "latitude", "lng_field": "longitude", "title_field": "title"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- LISTING VIEWS
-- =====================================================
IF et_listing IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_listing, 'listing_table', 'Table', 'table', true, 
        '["channel", "listing_price", "status", "start_date", "end_date"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_listing, 'listing_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "channel_name", "card_subtitle_field": "listing_price"}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_listing, 'listing_calendar', 'Calendar', 'calendar', false, 
        '[]'::jsonb, '{"date_field": "start_date", "end_date_field": "end_date", "title_field": "channel_name"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- VIEWING VIEWS
-- =====================================================
IF et_viewing IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_viewing, 'viewing_table', 'Table', 'table', true, 
        '["scheduled_start", "status", "agent_id"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_viewing, 'viewing_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "scheduled_start"}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_viewing, 'viewing_calendar', 'Calendar', 'calendar', false, 
        '[]'::jsonb, '{"date_field": "scheduled_start", "end_date_field": "scheduled_end", "title_field": "scheduled_start"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- OFFER VIEWS
-- =====================================================
IF et_offer IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_offer, 'offer_table', 'Table', 'table', true, 
        '["amount", "status", "submitted_at", "expires_at"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_offer, 'offer_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "amount", "card_subtitle_field": "submitted_at"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- CONTRACT VIEWS
-- =====================================================
IF et_contract IS NOT NULL THEN
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_contract, 'contract_table', 'Table', 'table', true, 
        '["contract_number", "contract_type", "amount", "status", "start_date", "end_date"]'::jsonb, '{}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_contract, 'contract_kanban', 'By Status', 'kanban', false, 
        '[]'::jsonb, '{"group_by_field": "status", "card_title_field": "contract_number", "card_subtitle_field": "amount"}'::jsonb)
    ON CONFLICT DO NOTHING;

    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, columns, settings)
    VALUES (gen_random_uuid(), tenant, et_contract, 'contract_calendar', 'Timeline', 'calendar', false, 
        '[]'::jsonb, '{"date_field": "start_date", "end_date_field": "end_date", "title_field": "contract_number"}'::jsonb)
    ON CONFLICT DO NOTHING;
END IF;

END $$;
