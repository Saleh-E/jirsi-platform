-- Phase 2 Task P2-A01: Listing Entity
-- Creates Listing entity for property marketing across channels

-- ============================================================================
-- LISTING ENTITY TYPE
-- ============================================================================

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
VALUES (
    'e0000000-0000-0000-0000-000000000020',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'listing',
    'Listing',
    'Listings',
    'megaphone'
)
ON CONFLICT (tenant_id, name) DO NOTHING;

-- ============================================================================
-- LISTING FIELD DEFINITIONS (12 fields)
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
-- Basic
('f4000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL),
('f4000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'channel', 'Channel', 'select', true, true, true, false, 2, 'Basic', NULL),
('f4000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'channel_name', 'Channel Name', 'text', false, true, false, false, 3, 'Basic', 'Bayut, Property Finder...'),
('f4000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'external_url', 'External URL', 'url', false, false, false, false, 4, 'Basic', NULL),

-- Pricing
('f4000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'listing_price', 'Listing Price', 'currency', false, true, true, false, 5, 'Pricing', NULL),
('f4000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'listing_currency', 'Currency', 'select', false, false, false, false, 6, 'Pricing', NULL),

-- Schedule
('f4000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'start_date', 'Start Date', 'date', true, true, false, false, 7, 'Schedule', NULL),
('f4000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'end_date', 'End Date', 'date', false, true, false, false, 8, 'Schedule', NULL),

-- Status
('f4000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'status', 'Status', 'select', true, true, true, false, 9, 'Status', NULL),

-- Details
('f4000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'description', 'Description', 'longtext', false, false, false, false, 10, 'Details', NULL),
('f4000001-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'virtual_tour_url', 'Virtual Tour URL', 'url', false, false, false, false, 11, 'Details', NULL),
('f4000001-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 'featured', 'Featured', 'boolean', false, true, false, false, 12, 'Details', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- Explicitly set lookup target for Property
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb WHERE id = 'f4000001-0000-0000-0000-000000000001';

-- ============================================================================
-- SELECT OPTIONS
-- ============================================================================

-- Channel Options
UPDATE field_defs 
SET options = '[
    {"value": "portal", "label": "Property Portal"},
    {"value": "website", "label": "Company Website"},
    {"value": "agent_network", "label": "Agent Network"},
    {"value": "social", "label": "Social Media"},
    {"value": "print", "label": "Print Media"},
    {"value": "direct", "label": "Direct Marketing"}
]'::jsonb
WHERE id = 'f4000001-0000-0000-0000-000000000002';

-- Currency Options
UPDATE field_defs 
SET options = '[
    {"value": "USD", "label": "USD ($)"},
    {"value": "AED", "label": "AED (د.إ)"},
    {"value": "EUR", "label": "EUR (€)"},
    {"value": "GBP", "label": "GBP (£)"},
    {"value": "SAR", "label": "SAR (ر.س)"}
]'::jsonb
WHERE id = 'f4000001-0000-0000-0000-000000000006';

-- Status Options
UPDATE field_defs 
SET options = '[
    {"value": "draft", "label": "Draft"},
    {"value": "pending", "label": "Pending Approval"},
    {"value": "active", "label": "Active"},
    {"value": "paused", "label": "Paused"},
    {"value": "expired", "label": "Expired"},
    {"value": "sold", "label": "Sold"},
    {"value": "rented", "label": "Rented"},
    {"value": "withdrawn", "label": "Withdrawn"}
]'::jsonb
WHERE id = 'f4000001-0000-0000-0000-000000000009';

-- ============================================================================
-- LISTINGS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    property_id UUID NOT NULL REFERENCES entity_records(id),
    
    -- Channel
    channel VARCHAR(50),
    channel_name VARCHAR(255),
    external_url TEXT,
    
    -- Pricing
    listing_price DECIMAL(15, 2),
    listing_currency VARCHAR(3) DEFAULT 'AED',
    
    -- Schedule
    start_date DATE NOT NULL,
    end_date DATE,
    
    -- Status
    status VARCHAR(30) DEFAULT 'draft',
    
    -- Details
    description TEXT,
    virtual_tour_url TEXT,
    featured BOOLEAN DEFAULT false,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_listings_tenant ON listings(tenant_id);
CREATE INDEX idx_listings_property ON listings(property_id);
CREATE INDEX idx_listings_status ON listings(status);
CREATE INDEX idx_listings_channel ON listings(channel);
CREATE INDEX idx_listings_dates ON listings(start_date, end_date);
CREATE INDEX idx_listings_featured ON listings(featured) WHERE featured = true;

-- ============================================================================
-- ASSOCIATION DEFS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
('adef0004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'listing_property', 'listing', 'property', 'Property', 'Listings', 'many_to_one', true, false)
ON CONFLICT (id) DO NOTHING;
