-- Phase 3 Task P3-03: Offer Entity Refinement
-- Updates Offer entity with finance_type and refined status options

-- ============================================================================
-- UPDATE OFFER TABLE STRUCTURE
-- ============================================================================

-- Add finance_type column if not exists
ALTER TABLE offers ADD COLUMN IF NOT EXISTS finance_type VARCHAR(50);

-- ============================================================================
-- UPDATE/INSERT FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder, options)
VALUES
('f8000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL, NULL),
('f8000003-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'contact_id', 'Buyer/Offerer', 'lookup', true, true, true, false, 2, 'Basic', NULL, NULL),
('f8000003-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'offer_price', 'Offer Price', 'currency', true, true, true, false, 3, 'Financial', NULL, NULL),
('f8000003-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'currency', 'Currency', 'select', true, true, false, false, 4, 'Financial', NULL,
 '[{"value": "AED", "label": "AED"}, {"value": "USD", "label": "USD"}, {"value": "EUR", "label": "EUR"}, {"value": "GBP", "label": "GBP"}, {"value": "SAR", "label": "SAR"}]'::jsonb),
('f8000003-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'finance_type', 'Finance Type', 'select', true, true, true, false, 5, 'Financial', NULL,
 '[{"value": "cash", "label": "Cash"}, {"value": "mortgage", "label": "Mortgage"}, {"value": "other", "label": "Other"}]'::jsonb),
('f8000003-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'status', 'Status', 'select', true, true, true, false, 6, 'Status', NULL,
 '[{"value": "new", "label": "New"}, {"value": "countered", "label": "Countered"}, {"value": "accepted", "label": "Accepted"}, {"value": "rejected", "label": "Rejected"}, {"value": "withdrawn", "label": "Withdrawn"}]'::jsonb),
('f8000003-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 
 'notes', 'Notes', 'textarea', false, false, false, false, 7, 'Details', NULL, NULL)
ON CONFLICT (entity_type_id, name) DO UPDATE SET
    label = EXCLUDED.label,
    field_type = EXCLUDED.field_type,
    is_required = EXCLUDED.is_required,
    show_in_list = EXCLUDED.show_in_list,
    sort_order = EXCLUDED.sort_order,
    options = EXCLUDED.options;

-- ============================================================================
-- UPDATE DEFAULT VIEWS FOR OFFER
-- ============================================================================

-- Update table view columns
UPDATE view_defs 
SET columns = '["property_id", "contact_id", "offer_price", "currency", "finance_type", "status"]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000012' AND view_type = 'table';

-- Kanban view by status
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012',
 'offers_pipeline', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "offer_price", "card_subtitle_field": "contact_id", "card_fields": ["property_id", "finance_type"], "column_order": ["new", "countered", "accepted", "rejected", "withdrawn"]}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- ============================================================================
-- CREATE INDEXES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_offers_finance_type ON offers(finance_type);
