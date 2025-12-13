-- Phase 2 Task P2-A03: Offer Enhancement
-- Adds negotiation tracking, deal linking, and commission fields to offers

-- ============================================================================
-- ADD MISSING COLUMNS TO OFFERS TABLE
-- ============================================================================

ALTER TABLE offers ADD COLUMN IF NOT EXISTS offer_type VARCHAR(50);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS negotiation_notes TEXT;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS final_amount DECIMAL(15, 2);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS commission_amount DECIMAL(15, 2);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS linked_deal_id UUID REFERENCES deals(id);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS deposit_amount DECIMAL(15, 2);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS financing_type VARCHAR(50);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS counter_offer_count INTEGER DEFAULT 0;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS accepted_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS rejected_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS rejection_reason TEXT;

-- ============================================================================
-- ADD NEW FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f3000002-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'offer_type', 'Offer Type', 'select', false, true, true, false, 10, 'Type', NULL),
('f3000002-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'negotiation_notes', 'Negotiation Notes', 'longtext', false, false, false, false, 11, 'Negotiation', NULL),
('f3000002-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'final_amount', 'Final Amount', 'currency', false, true, false, false, 12, 'Financial', NULL),
('f3000002-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'commission_amount', 'Commission', 'currency', false, false, false, false, 13, 'Financial', NULL),
('f3000002-0000-0000-0000-000000000014', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'linked_deal_id', 'Linked Deal', 'lookup', false, false, false, false, 14, 'Relations', NULL),
('f3000002-0000-0000-0000-000000000015', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'deposit_amount', 'Deposit', 'currency', false, false, false, false, 15, 'Financial', NULL),
('f3000002-0000-0000-0000-000000000016', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'financing_type', 'Financing Type', 'select', false, true, false, false, 16, 'Financial', NULL),
('f3000002-0000-0000-0000-000000000017', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'counter_offer_count', 'Counter Offers', 'integer', false, false, false, true, 17, 'Negotiation', NULL),
('f3000002-0000-0000-0000-000000000018', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'accepted_at', 'Accepted At', 'datetime', false, false, false, true, 18, 'Status', NULL),
('f3000002-0000-0000-0000-000000000019', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'rejected_at', 'Rejected At', 'datetime', false, false, false, true, 19, 'Status', NULL),
('f3000002-0000-0000-0000-000000000020', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000012', 'rejection_reason', 'Rejection Reason', 'text', false, false, false, false, 20, 'Status', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- SELECT OPTIONS FOR NEW FIELDS
-- ============================================================================

-- Offer Type Options
UPDATE field_defs 
SET options = '[
    {"value": "purchase", "label": "Purchase"},
    {"value": "rent", "label": "Rent/Lease"},
    {"value": "lease_to_own", "label": "Lease to Own"}
]'::jsonb
WHERE id = 'f3000002-0000-0000-0000-000000000010';

-- Financing Type Options
UPDATE field_defs 
SET options = '[
    {"value": "cash", "label": "Cash"},
    {"value": "mortgage", "label": "Mortgage/Bank Loan"},
    {"value": "developer_finance", "label": "Developer Finance"},
    {"value": "owner_finance", "label": "Owner Finance"},
    {"value": "mixed", "label": "Mixed Financing"}
]'::jsonb
WHERE id = 'f3000002-0000-0000-0000-000000000016';

-- ============================================================================
-- OFFER ASSOCIATIONS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
-- Offer → Property
('adef0006-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_property', 'offer', 'property', 'Property', 'Offers', 'many_to_one', true, false),
-- Offer → Contact (buyer)
('adef0006-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_buyer', 'offer', 'contact', 'Buyer', 'Offers Made', 'many_to_one', true, false),
-- Offer → Deal
('adef0006-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_deal', 'offer', 'deal', 'Deal', 'Offers', 'many_to_one', false, false),
-- Offer → Agent
('adef0006-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_agent', 'offer', 'contact', 'Agent', 'Handled Offers', 'many_to_one', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- CREATE INDEXES FOR PERFORMANCE
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_offers_linked_deal ON offers(linked_deal_id);
CREATE INDEX IF NOT EXISTS idx_offers_offer_type ON offers(offer_type);
CREATE INDEX IF NOT EXISTS idx_offers_financing ON offers(financing_type);
