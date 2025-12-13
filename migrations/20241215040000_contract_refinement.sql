-- Phase 3 Task P3-04: Contract Entity Refinement
-- Updates Contract entity with buyer/seller contacts and payment details

-- ============================================================================
-- UPDATE/INSERT FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder, options)
VALUES
('f8000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL, NULL),
('f8000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'buyer_contact_id', 'Buyer', 'lookup', true, true, true, false, 2, 'Parties', NULL, NULL),
('f8000004-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'seller_contact_id', 'Seller', 'lookup', true, true, true, false, 3, 'Parties', NULL, NULL),
('f8000004-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'contract_type', 'Type', 'select', true, true, true, false, 4, 'Basic', NULL,
 '[{"value": "sale", "label": "Sale"}, {"value": "rent", "label": "Rent"}]'::jsonb),
('f8000004-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'start_date', 'Start Date', 'date', true, true, true, false, 5, 'Dates', NULL, NULL),
('f8000004-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'end_date', 'End Date', 'date', false, true, false, false, 6, 'Dates', NULL, NULL),
('f8000004-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'contract_value', 'Contract Value', 'currency', true, true, true, false, 7, 'Financial', NULL, NULL),
('f8000004-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'deposit', 'Deposit', 'currency', false, true, false, false, 8, 'Financial', NULL, NULL),
('f8000004-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'payment_frequency', 'Payment Frequency', 'select', false, true, false, false, 9, 'Financial', NULL,
 '[{"value": "onetime", "label": "One-Time"}, {"value": "monthly", "label": "Monthly"}, {"value": "quarterly", "label": "Quarterly"}, {"value": "yearly", "label": "Yearly"}]'::jsonb),
('f8000004-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 
 'status', 'Status', 'select', true, true, true, false, 10, 'Status', NULL,
 '[{"value": "draft", "label": "Draft"}, {"value": "active", "label": "Active"}, {"value": "completed", "label": "Completed"}, {"value": "terminated", "label": "Terminated"}]'::jsonb)
ON CONFLICT (entity_type_id, name) DO UPDATE SET
    label = EXCLUDED.label,
    field_type = EXCLUDED.field_type,
    is_required = EXCLUDED.is_required,
    show_in_list = EXCLUDED.show_in_list,
    sort_order = EXCLUDED.sort_order,
    options = EXCLUDED.options;

-- ============================================================================
-- UPDATE DEFAULT VIEWS FOR CONTRACT
-- ============================================================================

-- Update table view columns
UPDATE view_defs 
SET columns = '["property_id", "buyer_contact_id", "seller_contact_id", "contract_type", "start_date", "end_date", "contract_value", "payment_frequency", "status"]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000021' AND view_type = 'table';

-- Calendar view for contracts (shows duration)
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021',
 'contracts_calendar', 'Calendar', 'calendar', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"date_field": "start_date", "end_date_field": "end_date", "title_field": "property_id", "color_field": "status", "color_map": {"draft": "#6b7280", "active": "#22c55e", "completed": "#3b82f6", "terminated": "#ef4444"}}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- Kanban view by status
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021',
 'contracts_pipeline', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "contract_value", "card_subtitle_field": "property_id", "card_fields": ["contract_type", "start_date", "buyer_contact_id"], "column_order": ["draft", "active", "completed", "terminated"]}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;
