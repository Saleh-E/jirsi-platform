-- Phase 2 Task P2-B01: ViewDef Saved Views (Seed)
-- Seeds default views for Real Estate entities
-- Schema: id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings

-- ============================================================================
-- PROPERTY VIEWS
-- ============================================================================

-- Property Table View (default)
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000003-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000010',
    'all_properties',
    'All Properties',
    'table',
    true,
    true,
    '["reference", "title", "property_type", "usage", "status", "city", "bedrooms", "price"]'::jsonb,
    '[]'::jsonb,
    '[{"field": "created_at", "desc": true}]'::jsonb,
    '{}'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Property Kanban View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000003-0000-0000-0000-000000000002',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000010',
    'property_kanban',
    'Pipeline',
    'kanban',
    false,
    true,
    '[]'::jsonb,
    '[]'::jsonb,
    '[]'::jsonb,
    '{
        "group_by_field": "status",
        "card_title_field": "title",
        "card_subtitle_field": "city",
        "card_fields": ["price", "bedrooms", "property_type"],
        "column_order": ["draft", "active", "reserved", "under_offer", "sold", "rented", "withdrawn"]
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Property Map View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000003-0000-0000-0000-000000000003',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000010',
    'property_map',
    'Map',
    'map',
    false,
    true,
    '[]'::jsonb,
    '[]'::jsonb,
    '[]'::jsonb,
    '{
        "lat_field": "latitude",
        "lng_field": "longitude",
        "popup_title_field": "title",
        "popup_fields": ["price", "bedrooms", "status"],
        "marker_color_field": "status",
        "default_center": [25.2048, 55.2708],
        "default_zoom": 11
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- VIEWING VIEWS
-- ============================================================================

-- Viewing Table View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000004-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000011',
    'all_viewings',
    'All Viewings',
    'table',
    true,
    true,
    '["property_id", "contact_id", "scheduled_at", "status", "duration_minutes"]'::jsonb,
    '[]'::jsonb,
    '[{"field": "scheduled_at", "desc": true}]'::jsonb,
    '{}'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Viewing Calendar View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000004-0000-0000-0000-000000000002',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000011',
    'viewings_calendar',
    'Calendar',
    'calendar',
    false,
    true,
    '[]'::jsonb,
    '[]'::jsonb,
    '[]'::jsonb,
    '{
        "date_field": "scheduled_at",
        "title_field": "property_id",
        "color_field": "status",
        "color_map": {
            "scheduled": "#3b82f6",
            "confirmed": "#22c55e",
            "completed": "#6b7280",
            "cancelled": "#ef4444",
            "no_show": "#f59e0b"
        },
        "default_view": "week"
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- LISTING VIEWS
-- ============================================================================

-- Listing Table View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000005-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000020',
    'all_listings',
    'All Listings',
    'table',
    true,
    true,
    '["property_id", "channel", "channel_name", "listing_price", "start_date", "end_date", "status", "featured"]'::jsonb,
    '[]'::jsonb,
    '[{"field": "created_at", "desc": true}]'::jsonb,
    '{}'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Listing Kanban View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000005-0000-0000-0000-000000000002',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000020',
    'listings_kanban',
    'Pipeline',
    'kanban',
    false,
    true,
    '[]'::jsonb,
    '[]'::jsonb,
    '[]'::jsonb,
    '{
        "group_by_field": "status",
        "card_title_field": "property_id",
        "card_subtitle_field": "channel_name",
        "card_fields": ["listing_price", "start_date"],
        "column_order": ["draft", "pending", "active", "paused", "expired", "sold", "rented", "withdrawn"]
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- OFFER VIEWS
-- ============================================================================

-- Offer Table View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000006-0000-0000-0000-000000000001',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000012',
    'all_offers',
    'All Offers',
    'table',
    true,
    true,
    '["property_id", "contact_id", "offer_amount", "currency", "status", "submitted_at"]'::jsonb,
    '[]'::jsonb,
    '[{"field": "created_at", "desc": true}]'::jsonb,
    '{}'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Offer Kanban View
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES (
    'a0000006-0000-0000-0000-000000000002',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000012',
    'offers_kanban',
    'Pipeline',
    'kanban',
    false,
    true,
    '[]'::jsonb,
    '[]'::jsonb,
    '[]'::jsonb,
    '{
        "group_by_field": "status",
        "card_title_field": "property_id",
        "card_subtitle_field": "contact_id",
        "card_fields": ["offer_amount", "submitted_at"],
        "column_order": ["draft", "submitted", "under_review", "countered", "accepted", "rejected", "withdrawn", "expired"]
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;
