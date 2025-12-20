-- Phase 3 Task P3-01: Listing Entity Refinement
-- Updates Listing entity to match required schema

-- ============================================================================
-- UPDATE LISTING TABLE STRUCTURE
-- ============================================================================

-- Add new columns to existing listings table
ALTER TABLE listings ADD COLUMN IF NOT EXISTS promo_price DECIMAL(15, 2);
ALTER TABLE listings ADD COLUMN IF NOT EXISTS headline VARCHAR(255);
ALTER TABLE listings ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE listings ADD COLUMN IF NOT EXISTS go_live_date DATE;
ALTER TABLE listings ADD COLUMN IF NOT EXISTS expiry_date DATE;

-- Rename existing columns if needed (list_price already exists as listing_price)
-- Ensure listing_status uses the correct enum values

-- ============================================================================
-- UPDATE/INSERT FIELD DEFINITIONS
-- ============================================================================

-- First, get the listing entity_type_id
-- Using existing ID: e0000000-0000-0000-0000-000000000020

-- Update existing field definitions and add new ones
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder, options)
VALUES
-- Required fields
('f8000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL, NULL),
('f8000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'channel', 'Channel', 'select', true, true, true, false, 2, 'Basic', NULL, 
 '[{"value": "website", "label": "Website"}, {"value": "portal", "label": "Portal"}, {"value": "social", "label": "Social Media"}, {"value": "agent", "label": "Agent Network"}, {"value": "other", "label": "Other"}]'::jsonb),
('f8000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'list_price', 'List Price', 'currency', true, true, true, false, 3, 'Pricing', NULL, NULL),
('f8000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'promo_price', 'Promotional Price', 'currency', false, true, false, false, 4, 'Pricing', NULL, NULL),
('f8000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'headline', 'Headline', 'text', true, true, true, false, 5, 'Content', 'Stunning villa with sea views...', NULL),
('f8000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'description', 'Description', 'longtext', false, false, false, false, 6, 'Content', NULL, NULL),
('f8000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'listing_status', 'Status', 'select', true, true, true, false, 7, 'Status', NULL,
 '[{"value": "draft", "label": "Draft"}, {"value": "live", "label": "Live"}, {"value": "paused", "label": "Paused"}, {"value": "expired", "label": "Expired"}]'::jsonb),
('f8000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'go_live_date', 'Go Live Date', 'date', false, true, false, false, 8, 'Dates', NULL, NULL),
('f8000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020', 
 'expiry_date', 'Expiry Date', 'date', false, true, false, false, 9, 'Dates', NULL, NULL)
ON CONFLICT (entity_type_id, name) DO UPDATE SET
    label = EXCLUDED.label,
    field_type = EXCLUDED.field_type,
    is_required = EXCLUDED.is_required,
    show_in_list = EXCLUDED.show_in_list,
    show_in_card = EXCLUDED.show_in_card,
    sort_order = EXCLUDED.sort_order,
    "group" = EXCLUDED."group",
    options = EXCLUDED.options;

-- ============================================================================
-- UPDATE DEFAULT VIEWS FOR LISTING
-- ============================================================================

-- Update table view columns
UPDATE view_defs 
SET columns = '["property_id", "channel", "headline", "list_price", "promo_price", "listing_status", "go_live_date", "expiry_date"]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000020' AND view_type = 'table';

-- Update/Insert Kanban view by listing_status
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020',
 'listings_pipeline', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "listing_status", "card_title_field": "headline", "card_subtitle_field": "property_id", "card_fields": ["list_price", "channel", "go_live_date"], "column_order": ["draft", "live", "paused", "expired"]}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- Card view for Listings
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000020',
 'listings_cards', 'Cards', 'card', false, true,
 '["headline", "list_price", "channel", "listing_status"]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"title_field": "headline", "subtitle_field": "channel", "image_field": null}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- ============================================================================
-- CREATE INDEXES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_listings_status ON listings(status);
CREATE INDEX IF NOT EXISTS idx_listings_channel ON listings(channel);
CREATE INDEX IF NOT EXISTS idx_listings_go_live ON listings(go_live_date);
CREATE INDEX IF NOT EXISTS idx_listings_expiry ON listings(expiry_date);
