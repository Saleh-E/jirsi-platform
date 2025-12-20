-- Phase 2 Task P2-A02: Viewing Enhancement
-- Adds calendar integration fields to viewing entity

-- ============================================================================
-- ADD MISSING FIELDS TO VIEWINGS TABLE
-- ============================================================================

ALTER TABLE viewings ADD COLUMN IF NOT EXISTS confirmed_at TIMESTAMPTZ;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMPTZ;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS cancelled_reason TEXT;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS reminder_sent BOOLEAN DEFAULT false;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS outcome VARCHAR(50);

-- ============================================================================
-- ADD NEW FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f2000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'confirmed_at', 'Confirmed At', 'datetime', false, false, false, true, 10, 'Status', NULL),
('f2000001-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'cancelled_at', 'Cancelled At', 'datetime', false, false, false, true, 11, 'Status', NULL),
('f2000001-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'cancelled_reason', 'Cancellation Reason', 'text', false, false, false, false, 12, 'Status', NULL),
('f2000001-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'reminder_sent', 'Reminder Sent', 'boolean', false, false, false, true, 13, 'Status', NULL),
('f2000001-0000-0000-0000-000000000014', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'outcome', 'Outcome', 'select', false, true, false, false, 14, 'Status', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- Outcome Options
UPDATE field_defs 
SET options = '[
    {"value": "interested", "label": "Interested"},
    {"value": "offer_made", "label": "Made Offer"},
    {"value": "not_interested", "label": "Not Interested"},
    {"value": "thinking", "label": "Still Thinking"},
    {"value": "needs_second", "label": "Needs Second Viewing"}
]'::jsonb
WHERE id = 'f2000001-0000-0000-0000-000000000014';

-- ============================================================================
-- ADD VIEWING ASSOCIATIONS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
-- Viewing → Property
('adef0005-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_property', 'viewing', 'property', 'Property', 'Viewings', 'many_to_one', true, false),
-- Viewing → Contact (viewer)
('adef0005-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_contact', 'viewing', 'contact', 'Contact', 'Viewings', 'many_to_one', true, false),
-- Viewing → Agent
('adef0005-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_agent', 'viewing', 'contact', 'Agent', 'Assigned Viewings', 'many_to_one', false, false)
ON CONFLICT (id) DO NOTHING;
