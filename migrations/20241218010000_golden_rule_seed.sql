-- Golden Rule Bible Definitions
-- Seed Property and Deal entities with spec-compliant field definitions
-- This follows the Golden Rule: Fields defined once, used everywhere

DO $$
DECLARE
    tenant UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
    et_property UUID;
    et_deal UUID;
    et_contact UUID;
BEGIN

-- Get entity type IDs
SELECT id INTO et_property FROM entity_types WHERE tenant_id = tenant AND name = 'property' LIMIT 1;
SELECT id INTO et_deal FROM entity_types WHERE tenant_id = tenant AND name = 'deal' LIMIT 1;
SELECT id INTO et_contact FROM entity_types WHERE tenant_id = tenant AND name = 'contact' LIMIT 1;

-- =====================================================
-- PROPERTY ENTITY - Bible Fields
-- =====================================================
IF et_property IS NOT NULL THEN
    -- Sale Price (Money with USD)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, options, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_property, 'sale_price', 'Sale Price', 'money', false, true, true, true, 
        '{"currency": "USD"}'::jsonb, 10, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Status (Select with Draft/Active/Sold)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, options, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_property, 'property_status', 'Status', 'select', false, true, true, true, 
        '{"choices": [{"value": "draft", "label": "Draft", "color": "#6c757d"}, {"value": "active", "label": "Active", "color": "#28a745"}, {"value": "sold", "label": "Sold", "color": "#dc3545"}]}'::jsonb, 20, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Primary Image
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_property, 'primary_image', 'Image', 'attachment', false, true, true, 30, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Property Views (Table, Map, Kanban)
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, settings, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_property, 'all_properties', 'All Properties', 'table', true, true,
        '["title", "property_type", "sale_price", "property_status", "city"]'::jsonb, '{}'::jsonb, now(), now())
    ON CONFLICT DO NOTHING;
    
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, settings, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_property, 'property_map', 'Property Map', 'map', false, true,
        '["title", "sale_price"]'::jsonb, 
        '{"lat_field": "latitude", "lng_field": "longitude", "title_field": "title"}'::jsonb, now(), now())
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- DEAL ENTITY - Bible Fields
-- =====================================================
IF et_deal IS NOT NULL THEN
    -- Amount (Money)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, options, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_deal, 'deal_amount', 'Amount', 'money', false, true, true, true, 
        '{"currency": "USD"}'::jsonb, 10, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Stage (Select with pipeline stages)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, options, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_deal, 'deal_stage', 'Stage', 'select', false, true, true, true, 
        '{"choices": [{"value": "new", "label": "New", "color": "#17a2b8"}, {"value": "negotiation", "label": "Negotiation", "color": "#ffc107"}, {"value": "won", "label": "Won", "color": "#28a745"}, {"value": "lost", "label": "Lost", "color": "#dc3545"}]}'::jsonb, 20, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Closing Date
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_deal, 'closing_date', 'Closing Date', 'date', false, true, true, true, 30, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Sales Pipeline (Kanban View)
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, settings, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_deal, 'sales_pipeline', 'Sales Pipeline', 'kanban', false, true,
        '["name", "deal_amount", "closing_date"]'::jsonb, 
        '{"group_by_field": "stage", "card_title_field": "name", "card_subtitle_field": "deal_amount"}'::jsonb, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- All Deals Table View
    INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, settings, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_deal, 'all_deals', 'All Deals', 'table', true, true,
        '["name", "deal_amount", "deal_stage", "closing_date"]'::jsonb, '{}'::jsonb, now(), now())
    ON CONFLICT DO NOTHING;
END IF;

-- =====================================================
-- CONTACT ENTITY - Additional Bible Fields
-- =====================================================
IF et_contact IS NOT NULL THEN
    -- Lead Score (Score field)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, is_sortable, options, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_contact, 'lead_score', 'Lead Score', 'score', false, true, true, true, 
        '{"max_score": 100}'::jsonb, 50, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Tags (TagList field)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, is_filterable, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_contact, 'tags', 'Tags', 'tag_list', false, true, true, 60, now(), now())
    ON CONFLICT DO NOTHING;
    
    -- Avatar/Photo (Image field)
    INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, sort_order, created_at, updated_at)
    VALUES (gen_random_uuid(), tenant, et_contact, 'avatar', 'Avatar', 'attachment', false, false, true, 70, now(), now())
    ON CONFLICT DO NOTHING;
END IF;

END $$;
