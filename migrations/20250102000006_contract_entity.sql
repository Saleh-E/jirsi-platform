-- Phase 2 Task P2-A04: Contract/Lease Entity
-- Creates contract entity for sales agreements and rental leases

-- ============================================================================
-- CONTRACT ENTITY TYPE
-- ============================================================================

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
VALUES (
    'e0000000-0000-0000-0000-000000000021',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'realestate',
    'contract',
    'Contract',
    'Contracts',
    'file-signature'
)
ON CONFLICT (tenant_id, name) DO NOTHING;

-- ============================================================================
-- CONTRACT FIELD DEFINITIONS (20 fields)
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
-- Basic
('f5000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'contract_number', 'Contract Number', 'text', true, true, true, false, 1, 'Basic', 'CNT-001'),
('f5000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'contract_type', 'Contract Type', 'select', true, true, true, false, 2, 'Basic', NULL),
('f5000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'property_id', 'Property', 'lookup', true, true, true, false, 3, 'Basic', NULL),
('f5000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'offer_id', 'Linked Offer', 'lookup', false, false, false, false, 4, 'Basic', NULL),

-- Parties (for sale)
('f5000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'seller_id', 'Seller', 'lookup', false, true, false, false, 5, 'Parties', NULL),
('f5000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'buyer_id', 'Buyer', 'lookup', false, true, false, false, 6, 'Parties', NULL),

-- Parties (for rent)
('f5000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'landlord_id', 'Landlord', 'lookup', false, false, false, false, 7, 'Parties', NULL),
('f5000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'tenant_id', 'Tenant', 'lookup', false, false, false, false, 8, 'Parties', NULL),
('f5000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'agent_id', 'Agent', 'lookup', false, true, false, false, 9, 'Parties', NULL),

-- Terms
('f5000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'start_date', 'Start Date', 'date', true, true, true, false, 10, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'end_date', 'End Date', 'date', false, true, false, false, 11, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'amount', 'Amount', 'currency', true, true, true, false, 12, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'currency', 'Currency', 'select', false, false, false, false, 13, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000014', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'payment_frequency', 'Payment Frequency', 'select', false, true, false, false, 14, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000015', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'deposit_amount', 'Security Deposit', 'currency', false, false, false, false, 15, 'Terms', NULL),
('f5000001-0000-0000-0000-000000000016', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'commission_amount', 'Commission', 'currency', false, false, false, false, 16, 'Terms', NULL),

-- Status
('f5000001-0000-0000-0000-000000000017', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'status', 'Status', 'select', true, true, true, false, 17, 'Status', NULL),
('f5000001-0000-0000-0000-000000000018', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'signed_at', 'Signed At', 'datetime', false, false, false, false, 18, 'Status', NULL),
('f5000001-0000-0000-0000-000000000019', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'terminated_at', 'Terminated At', 'datetime', false, false, false, true, 19, 'Status', NULL),
('f5000001-0000-0000-0000-000000000020', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021', 'termination_reason', 'Termination Reason', 'text', false, false, false, false, 20, 'Status', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- Explicitly set lookup target for Property
UPDATE field_defs SET ui_hints = '{"lookup_entity": "property"}'::jsonb WHERE id = 'f5000001-0000-0000-0000-000000000003';

-- ============================================================================
-- SELECT OPTIONS
-- ============================================================================

-- Contract Type Options
UPDATE field_defs 
SET options = '[
    {"value": "sale", "label": "Sale Agreement"},
    {"value": "rent", "label": "Rental Agreement"},
    {"value": "lease", "label": "Long-term Lease"},
    {"value": "commercial", "label": "Commercial Lease"}
]'::jsonb
WHERE id = 'f5000001-0000-0000-0000-000000000002';

-- Currency Options
UPDATE field_defs 
SET options = '[
    {"value": "USD", "label": "USD ($)"},
    {"value": "AED", "label": "AED (د.إ)"},
    {"value": "EUR", "label": "EUR (€)"},
    {"value": "GBP", "label": "GBP (£)"},
    {"value": "SAR", "label": "SAR (ر.س)"}
]'::jsonb
WHERE id = 'f5000001-0000-0000-0000-000000000013';

-- Payment Frequency Options
UPDATE field_defs 
SET options = '[
    {"value": "one_time", "label": "One-time Payment"},
    {"value": "monthly", "label": "Monthly"},
    {"value": "quarterly", "label": "Quarterly"},
    {"value": "semi_annual", "label": "Semi-Annual"},
    {"value": "yearly", "label": "Yearly"}
]'::jsonb
WHERE id = 'f5000001-0000-0000-0000-000000000014';

-- Status Options
UPDATE field_defs 
SET options = '[
    {"value": "draft", "label": "Draft"},
    {"value": "pending_signature", "label": "Pending Signature"},
    {"value": "active", "label": "Active"},
    {"value": "completed", "label": "Completed"},
    {"value": "terminated", "label": "Terminated Early"},
    {"value": "expired", "label": "Expired"},
    {"value": "renewed", "label": "Renewed"}
]'::jsonb
WHERE id = 'f5000001-0000-0000-0000-000000000017';

-- ============================================================================
-- CONTRACTS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Basic
    contract_number VARCHAR(100) NOT NULL,
    contract_type VARCHAR(50) NOT NULL,
    property_id UUID NOT NULL REFERENCES entity_records(id),
    offer_id UUID REFERENCES offers(id),
    
    -- Parties (sale)
    seller_id UUID REFERENCES contacts(id),
    buyer_id UUID REFERENCES contacts(id),
    
    -- Parties (rent)
    landlord_id UUID REFERENCES contacts(id),
    tenant_contact_id UUID REFERENCES contacts(id),
    agent_id UUID REFERENCES contacts(id),
    
    -- Terms
    start_date DATE NOT NULL,
    end_date DATE,
    amount DECIMAL(15, 2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'AED',
    payment_frequency VARCHAR(30),
    deposit_amount DECIMAL(15, 2),
    commission_amount DECIMAL(15, 2),
    
    -- Status
    status VARCHAR(50) DEFAULT 'draft',
    signed_at TIMESTAMPTZ,
    terminated_at TIMESTAMPTZ,
    termination_reason TEXT,
    
    -- Notes
    notes TEXT,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    -- Unique contract number per tenant
    UNIQUE(tenant_id, contract_number)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_contracts_tenant ON contracts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_contracts_property ON contracts(property_id);
CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status);
CREATE INDEX IF NOT EXISTS idx_contracts_type ON contracts(contract_type);
CREATE INDEX IF NOT EXISTS idx_contracts_dates ON contracts(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_contracts_seller ON contracts(seller_id);
CREATE INDEX IF NOT EXISTS idx_contracts_buyer ON contracts(buyer_id);
CREATE INDEX IF NOT EXISTS idx_contracts_landlord ON contracts(landlord_id);
CREATE INDEX IF NOT EXISTS idx_contracts_tenant_contact ON contracts(tenant_contact_id);

-- ============================================================================
-- CONTRACT ASSOCIATIONS
-- ============================================================================

INSERT INTO association_defs (id, tenant_id, name, source_entity, target_entity, 
    label_source, label_target, cardinality, allow_primary, cascade_delete)
VALUES
-- Contract → Property
('adef0007-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_property', 'contract', 'property', 'Property', 'Contracts', 'many_to_one', true, false),
-- Contract → Offer
('adef0007-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_offer', 'contract', 'offer', 'Offer', 'Contract', 'many_to_one', false, false),
-- Contract → Seller
('adef0007-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_seller', 'contract', 'contact', 'Seller', 'Contracts (as Seller)', 'many_to_one', false, false),
-- Contract → Buyer
('adef0007-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_buyer', 'contract', 'contact', 'Buyer', 'Contracts (as Buyer)', 'many_to_one', false, false),
-- Contract → Landlord
('adef0007-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_landlord', 'contract', 'contact', 'Landlord', 'Contracts (as Landlord)', 'many_to_one', false, false),
-- Contract → Tenant
('adef0007-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_tenant', 'contract', 'contact', 'Tenant', 'Contracts (as Tenant)', 'many_to_one', false, false),
-- Contract → Agent
('adef0007-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_agent', 'contract', 'contact', 'Agent', 'Contracts (as Agent)', 'many_to_one', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- DEFAULT VIEWS FOR CONTRACT
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Contract Table View
('a0000007-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021',
 'all_contracts', 'All Contracts', 'table', true, true,
 '["contract_number", "contract_type", "property_id", "seller_id", "buyer_id", "start_date", "end_date", "amount", "status"]'::jsonb,
 '[]'::jsonb, '[{"field": "created_at", "desc": true}]'::jsonb, '{}'::jsonb),
-- Contract Kanban View
('a0000007-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021',
 'contracts_kanban', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "contract_number", "card_fields": ["amount", "start_date", "contract_type"]}'::jsonb),
-- Contract Calendar View (lease durations)
('a0000007-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000021',
 'contracts_calendar', 'Calendar', 'calendar', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"date_field": "start_date", "end_date_field": "end_date", "title_field": "contract_number", "color_field": "status"}'::jsonb)
ON CONFLICT (id) DO NOTHING;
