-- Migration Fixes: Consolidates all missing metadata
-- Fixes audit issues found in migration review
-- Created: 2025-01-02

-- ============================================================================
-- PART 1: Property Select Field Options
-- ============================================================================

-- Property Type Options
UPDATE field_defs 
SET options = '[
    {"value": "apartment", "label": "Apartment"},
    {"value": "villa", "label": "Villa"},
    {"value": "townhouse", "label": "Townhouse"},
    {"value": "penthouse", "label": "Penthouse"},
    {"value": "duplex", "label": "Duplex"},
    {"value": "studio", "label": "Studio"},
    {"value": "land", "label": "Land"},
    {"value": "commercial", "label": "Commercial"},
    {"value": "warehouse", "label": "Warehouse"},
    {"value": "office", "label": "Office"}
]'::jsonb
WHERE name = 'property_type' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- Property Usage Options
UPDATE field_defs 
SET options = '[
    {"value": "sale", "label": "For Sale"},
    {"value": "rent", "label": "For Rent"},
    {"value": "both", "label": "Sale or Rent"},
    {"value": "off_plan", "label": "Off-Plan"},
    {"value": "auction", "label": "Auction"}
]'::jsonb
WHERE name = 'usage' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- Property Status Options
UPDATE field_defs 
SET options = '[
    {"value": "draft", "label": "Draft", "color": "#94a3b8"},
    {"value": "available", "label": "Available", "color": "#22c55e"},
    {"value": "under_offer", "label": "Under Offer", "color": "#f59e0b"},
    {"value": "sold", "label": "Sold", "color": "#6366f1"},
    {"value": "rented", "label": "Rented", "color": "#8b5cf6"},
    {"value": "off_market", "label": "Off Market", "color": "#ef4444"},
    {"value": "coming_soon", "label": "Coming Soon", "color": "#06b6d4"}
]'::jsonb
WHERE name = 'status' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- Property Currency Options
UPDATE field_defs 
SET options = '[
    {"value": "USD", "label": "USD ($)"},
    {"value": "AED", "label": "AED (د.إ)"},
    {"value": "EUR", "label": "EUR (€)"},
    {"value": "GBP", "label": "GBP (£)"},
    {"value": "SAR", "label": "SAR (ر.س)"}
]'::jsonb
WHERE name = 'currency' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- Property Amenities Options  
UPDATE field_defs 
SET options = '[
    {"value": "pool", "label": "Swimming Pool"},
    {"value": "gym", "label": "Gym"},
    {"value": "parking", "label": "Parking"},
    {"value": "garden", "label": "Garden"},
    {"value": "balcony", "label": "Balcony"},
    {"value": "security", "label": "24/7 Security"},
    {"value": "concierge", "label": "Concierge"},
    {"value": "maid_room", "label": "Maid Room"},
    {"value": "study", "label": "Study"},
    {"value": "beach_access", "label": "Beach Access"},
    {"value": "golf_view", "label": "Golf View"},
    {"value": "sea_view", "label": "Sea View"},
    {"value": "city_view", "label": "City View"},
    {"value": "furnished", "label": "Furnished"},
    {"value": "pets_allowed", "label": "Pets Allowed"}
]'::jsonb
WHERE name = 'amenities' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- ============================================================================
-- PART 2: Default Views for Property
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Property Table View
('a0000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010',
 'all_properties', 'All Properties', 'table', true, true,
 '["reference", "title", "property_type", "usage", "status", "city", "bedrooms", "price"]'::jsonb,
 '[]'::jsonb, '[{"field": "created_at", "desc": true}]'::jsonb, '{}'::jsonb),
-- Property Kanban View
('a0000003-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010',
 'properties_kanban', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "title", "card_fields": ["price", "bedrooms", "city"]}'::jsonb),
-- Property Map View
('a0000003-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010',
 'properties_map', 'Map View', 'map', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"latitude_field": "latitude", "longitude_field": "longitude", "title_field": "title", "popup_fields": ["price", "bedrooms", "status"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 3: Default Views for Listing
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Listing Table View
('a0000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020',
 'all_listings', 'All Listings', 'table', true, true,
 '["property_id", "channel", "status", "listing_price", "start_date", "end_date", "featured"]'::jsonb,
 '[]'::jsonb, '[{"field": "created_at", "desc": true}]'::jsonb, '{}'::jsonb),
-- Listing Kanban View
('a0000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020',
 'listings_kanban', 'By Status', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "channel_name", "card_fields": ["listing_price", "start_date"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 4: Default Views for Viewing
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Viewing Table View
('a0000005-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'all_viewings', 'All Viewings', 'table', true, true,
 '["property_id", "contact_id", "agent_id", "scheduled_at", "status"]'::jsonb,
 '[]'::jsonb, '[{"field": "scheduled_at", "desc": true}]'::jsonb, '{}'::jsonb),
-- Viewing Calendar View
('a0000005-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'viewings_calendar', 'Calendar', 'calendar', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"date_field": "scheduled_at", "title_field": "property_id", "color_field": "status"}'::jsonb),
-- Viewing Kanban View
('a0000005-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'viewings_kanban', 'By Status', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "contact_id", "card_fields": ["scheduled_at", "agent_id"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 5: Default Views for Offer
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Offer Table View
('a0000006-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012',
 'all_offers', 'All Offers', 'table', true, true,
 '["property_id", "contact_id", "offer_amount", "status", "submitted_at"]'::jsonb,
 '[]'::jsonb, '[{"field": "submitted_at", "desc": true}]'::jsonb, '{}'::jsonb),
-- Offer Kanban View
('a0000006-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012',
 'offers_kanban', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "offer_amount", "card_fields": ["property_id", "contact_id", "submitted_at"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 6: Default Views for Contact
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('a0000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001',
 'all_contacts', 'All Contacts', 'table', true, true,
 '["first_name", "last_name", "email", "phone", "lifecycle_stage"]'::jsonb,
 '[]'::jsonb, '[{"field": "created_at", "desc": true}]'::jsonb, '{}'::jsonb),
('a0000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001',
 'contacts_kanban', 'By Stage', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "lifecycle_stage", "card_title_field": "first_name", "card_fields": ["email", "phone"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 7: Default Views for Company
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('a0000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000002',
 'all_companies', 'All Companies', 'table', true, true,
 '["name", "domain", "phone", "city"]'::jsonb,
 '[]'::jsonb, '[{"field": "created_at", "desc": true}]'::jsonb, '{}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 8: Default Views for Task
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('a0000008-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004',
 'all_tasks', 'All Tasks', 'table', true, true,
 '["title", "status", "priority", "due_date", "assignee_id"]'::jsonb,
 '[]'::jsonb, '[{"field": "due_date", "desc": false}]'::jsonb, '{}'::jsonb),
('a0000008-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004',
 'tasks_kanban', 'Kanban Board', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "title", "card_fields": ["priority", "due_date"]}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- PART 9: Add Task Priority and Status Options
-- ============================================================================

-- Task Priority Options
UPDATE field_defs 
SET options = '[
    {"value": "low", "label": "Low", "color": "#94a3b8"},
    {"value": "normal", "label": "Normal", "color": "#22c55e"},
    {"value": "high", "label": "High", "color": "#f59e0b"},
    {"value": "urgent", "label": "Urgent", "color": "#ef4444"}
]'::jsonb
WHERE name = 'priority' AND entity_type_id = 'e0000000-0000-0000-0000-000000000004';

-- ============================================================================
-- PART 10: Lookup Field UI Hints
-- ============================================================================

-- Set lookup targets for Contact fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "company"}'::jsonb 
WHERE name = 'company_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000001';

-- Set lookup targets for Property fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'owner_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'agent_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "company"}'::jsonb 
WHERE name = 'developer_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000010';

-- Set lookup targets for Deal fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "pipeline"}'::jsonb 
WHERE name = 'pipeline_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000003';

-- Set lookup targets for Task fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "user"}'::jsonb 
WHERE name = 'assignee_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000004';

-- Set lookup targets for Viewing fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'contact_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000011';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'agent_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000011';

-- Set lookup targets for Offer fields
UPDATE field_defs SET ui_hints = '{"lookup_entity": "contact"}'::jsonb 
WHERE name = 'contact_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000012';

UPDATE field_defs SET ui_hints = '{"lookup_entity": "deal"}'::jsonb 
WHERE name = 'deal_id' AND entity_type_id = 'e0000000-0000-0000-0000-000000000012';

-- ============================================================================
-- PART 11: Create activity_log table if not exists
-- ============================================================================

CREATE TABLE IF NOT EXISTS activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    changes JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_log_entity ON activity_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_tenant ON activity_log(tenant_id, created_at DESC);

-- ============================================================================
-- PART 12: Create properties table if not exists
-- ============================================================================

CREATE TABLE IF NOT EXISTS properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    reference VARCHAR(100),
    title VARCHAR(255),
    description TEXT,
    property_type VARCHAR(50),
    usage VARCHAR(50),
    status VARCHAR(50) DEFAULT 'draft',
    country VARCHAR(100),
    city VARCHAR(100),
    area VARCHAR(255),
    address TEXT,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    bedrooms INTEGER,
    bathrooms INTEGER,
    size_sqm DECIMAL(10, 2),
    floor INTEGER,
    total_floors INTEGER,
    year_built INTEGER,
    price DECIMAL(15, 2),
    rent_amount DECIMAL(15, 2),
    currency VARCHAR(3) DEFAULT 'AED',
    service_charge DECIMAL(15, 2),
    commission_percent DECIMAL(5, 2),
    owner_id UUID,
    agent_id UUID,
    developer_id UUID,
    amenities TEXT[],
    photos TEXT[],
    documents TEXT[],
    listed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_properties_tenant ON properties(tenant_id);
CREATE INDEX IF NOT EXISTS idx_properties_status ON properties(status);
CREATE INDEX IF NOT EXISTS idx_properties_type ON properties(property_type);

-- ============================================================================
-- COMPLETION
-- ============================================================================

-- Migration fixes applied successfully
