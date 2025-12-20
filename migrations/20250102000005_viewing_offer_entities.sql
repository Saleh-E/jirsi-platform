-- Phase 1B Tasks P1B-08/09: Viewing and Offer Entity Types
-- Creates entity definitions, fields, and tables for Viewing and Offer

-- ============================================================================
-- VIEWING ENTITY TYPE
-- ============================================================================

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
VALUES (
    'e0000000-0000-0000-0000-000000000011',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'viewing',
    'Viewing',
    'Viewings',
    'calendar'
)
ON CONFLICT (tenant_id, name) DO NOTHING;

-- Viewing Fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f2000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL),
('f2000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'contact_id', 'Contact', 'lookup', true, true, true, false, 2, 'Basic', NULL),
('f2000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'agent_id', 'Agent', 'lookup', false, true, false, false, 3, 'Basic', NULL),
('f2000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'scheduled_at', 'Scheduled At', 'datetime', true, true, true, false, 4, 'Schedule', NULL),
('f2000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'duration_minutes', 'Duration (min)', 'integer', false, false, false, false, 5, 'Schedule', NULL),
('f2000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'status', 'Status', 'select', true, true, true, false, 6, 'Status', NULL),
('f2000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'feedback', 'Feedback', 'textarea', false, false, false, false, 7, 'Details', NULL),
('f2000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'rating', 'Rating', 'integer', false, false, false, false, 8, 'Details', NULL),
('f2000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 'follow_up_notes', 'Follow-up Notes', 'textarea', false, false, false, false, 9, 'Details', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- Explicitly set lookup target for Property
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb WHERE id = 'f2000001-0000-0000-0000-000000000001';

-- Viewing Status Options
UPDATE field_defs 
SET options = '[
    {"value": "scheduled", "label": "Scheduled"},
    {"value": "confirmed", "label": "Confirmed"},
    {"value": "completed", "label": "Completed"},
    {"value": "cancelled", "label": "Cancelled"},
    {"value": "no_show", "label": "No Show"}
]'::jsonb
WHERE id = 'f2000001-0000-0000-0000-000000000006';

-- Viewings Table
CREATE TABLE IF NOT EXISTS viewings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    property_id UUID NOT NULL,
    contact_id UUID NOT NULL,
    agent_id UUID,
    scheduled_at TIMESTAMPTZ NOT NULL,
    duration_minutes INTEGER DEFAULT 30,
    status VARCHAR(30) DEFAULT 'scheduled',
    feedback TEXT,
    rating INTEGER,
    follow_up_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_viewings_tenant ON viewings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_viewings_property ON viewings(property_id);
CREATE INDEX IF NOT EXISTS idx_viewings_contact ON viewings(contact_id);
CREATE INDEX IF NOT EXISTS idx_viewings_scheduled ON viewings(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_viewings_status ON viewings(status);

-- ============================================================================
-- OFFER ENTITY TYPE
-- ============================================================================

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
VALUES (
    'e0000000-0000-0000-0000-000000000012',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'offer',
    'Offer',
    'Offers',
    'file-text'
)
ON CONFLICT (tenant_id, name) DO NOTHING;

-- Offer Fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f3000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL),
('f3000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'contact_id', 'Contact', 'lookup', true, true, true, false, 2, 'Basic', NULL),
('f3000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'deal_id', 'Deal', 'lookup', false, false, false, false, 3, 'Basic', NULL),
('f3000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'offer_amount', 'Offer Amount', 'currency', true, true, true, false, 4, 'Financial', NULL),
('f3000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'currency', 'Currency', 'select', true, true, false, false, 5, 'Financial', NULL),
('f3000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'status', 'Status', 'select', true, true, true, false, 6, 'Status', NULL),
('f3000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'submitted_at', 'Submitted At', 'datetime', true, true, false, false, 7, 'Dates', NULL),
('f3000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'expires_at', 'Expires At', 'datetime', false, false, false, false, 8, 'Dates', NULL),
('f3000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'conditions', 'Conditions', 'textarea', false, false, false, false, 9, 'Details', NULL),
('f3000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'counter_amount', 'Counter Amount', 'currency', false, false, false, false, 10, 'Financial', NULL),
('f3000001-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'accepted_at', 'Accepted At', 'datetime', false, false, false, true, 11, 'Dates', NULL),
('f3000001-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'rejected_reason', 'Rejection Reason', 'text', false, false, false, false, 12, 'Details', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- Explicitly set lookup target for Property
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb WHERE id = 'f3000001-0000-0000-0000-000000000001';

-- Offer Status Options
UPDATE field_defs 
SET options = '[
    {"value": "draft", "label": "Draft"},
    {"value": "submitted", "label": "Submitted"},
    {"value": "under_review", "label": "Under Review"},
    {"value": "countered", "label": "Countered"},
    {"value": "accepted", "label": "Accepted"},
    {"value": "rejected", "label": "Rejected"},
    {"value": "withdrawn", "label": "Withdrawn"},
    {"value": "expired", "label": "Expired"}
]'::jsonb
WHERE id = 'f3000001-0000-0000-0000-000000000006';

-- Offer Currency Options (same as property)
UPDATE field_defs 
SET options = '[
    {"value": "USD", "label": "USD ($)"},
    {"value": "AED", "label": "AED (د.إ)"},
    {"value": "EUR", "label": "EUR (€)"},
    {"value": "GBP", "label": "GBP (£)"},
    {"value": "SAR", "label": "SAR (ر.س)"}
]'::jsonb
WHERE id = 'f3000001-0000-0000-0000-000000000005';

-- Offers Table
CREATE TABLE IF NOT EXISTS offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    property_id UUID NOT NULL,
    contact_id UUID NOT NULL,
    deal_id UUID,
    offer_amount DECIMAL(15, 2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(30) DEFAULT 'draft',
    submitted_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    conditions TEXT,
    counter_amount DECIMAL(15, 2),
    accepted_at TIMESTAMPTZ,
    rejected_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_offers_tenant ON offers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_offers_property ON offers(property_id);
CREATE INDEX IF NOT EXISTS idx_offers_contact ON offers(contact_id);
CREATE INDEX IF NOT EXISTS idx_offers_status ON offers(status);
CREATE INDEX IF NOT EXISTS idx_offers_submitted ON offers(submitted_at);
