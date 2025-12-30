-- ============================================================================
-- Payment Infrastructure for Stripe Connect
-- Enables landlord payment collection and commission handling
-- ============================================================================

-- Payment Merchants (Landlords/Property Owners with Stripe Connect accounts)
CREATE TABLE IF NOT EXISTS payment_merchants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Link to landlord (contact or user)
    landlord_id UUID NOT NULL,
    landlord_type VARCHAR(50) NOT NULL CHECK (landlord_type IN ('contact', 'user')),
    
    -- Stripe Connect details
    stripe_account_id VARCHAR(100),
    stripe_account_status VARCHAR(50) DEFAULT 'pending' 
        CHECK (stripe_account_status IN ('pending', 'onboarding', 'active', 'restricted', 'disabled')),
    stripe_charges_enabled BOOLEAN DEFAULT FALSE,
    stripe_payouts_enabled BOOLEAN DEFAULT FALSE,
    stripe_default_currency VARCHAR(3) DEFAULT 'USD',
    
    -- Business verification (KYC)
    business_type VARCHAR(50) CHECK (business_type IN ('individual', 'company', 'non_profit')),
    verification_status VARCHAR(50) DEFAULT 'unverified'
        CHECK (verification_status IN ('unverified', 'pending', 'verified', 'failed')),
    verification_fields_needed JSONB DEFAULT '[]',
    
    -- Bank account info (for display, not sensitive)
    bank_name VARCHAR(255),
    bank_last_four VARCHAR(4),
    
    -- Commission settings
    default_commission_percent DECIMAL(5,2) DEFAULT 5.00,
    
    -- Metadata
    onboarding_completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, landlord_id)
);

CREATE INDEX IF NOT EXISTS idx_payment_merchants_tenant 
    ON payment_merchants(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payment_merchants_stripe 
    ON payment_merchants(stripe_account_id) WHERE stripe_account_id IS NOT NULL;

-- Payments table (linked to contracts)
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Links
    contract_id UUID,  -- Optional link to contract
    merchant_id UUID REFERENCES payment_merchants(id) ON DELETE SET NULL,
    payer_id UUID,     -- Contact who is paying
    
    -- Payment details
    amount DECIMAL(12,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    description TEXT,
    payment_type VARCHAR(50) NOT NULL 
        CHECK (payment_type IN ('rent', 'deposit', 'commission', 'fee', 'refund', 'other')),
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'succeeded', 'failed', 'refunded', 'cancelled')),
    
    -- Stripe details
    stripe_payment_intent_id VARCHAR(100),
    stripe_charge_id VARCHAR(100),
    stripe_transfer_id VARCHAR(100),  -- For Connect transfers
    
    -- Commission handling
    commission_amount DECIMAL(12,2) DEFAULT 0,
    commission_percent DECIMAL(5,2),
    net_amount DECIMAL(12,2),  -- Amount after commission
    
    -- Payment method used
    payment_method_type VARCHAR(50),  -- 'card', 'bank_transfer', 'cash'
    payment_method_last_four VARCHAR(4),
    
    -- Timing
    due_date DATE,
    paid_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payments_tenant 
    ON payments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payments_contract 
    ON payments(tenant_id, contract_id) WHERE contract_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_payments_merchant 
    ON payments(merchant_id);
CREATE INDEX IF NOT EXISTS idx_payments_status 
    ON payments(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_payments_stripe_intent 
    ON payments(stripe_payment_intent_id) WHERE stripe_payment_intent_id IS NOT NULL;

-- Payment schedule (recurring payments)
CREATE TABLE IF NOT EXISTS payment_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Links
    contract_id UUID NOT NULL,
    merchant_id UUID REFERENCES payment_merchants(id) ON DELETE SET NULL,
    payer_id UUID NOT NULL,
    
    -- Schedule details
    amount DECIMAL(12,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    frequency VARCHAR(20) NOT NULL 
        CHECK (frequency IN ('weekly', 'biweekly', 'monthly', 'quarterly', 'yearly', 'one_time')),
    
    -- Payment window
    day_of_month INTEGER CHECK (day_of_month BETWEEN 1 AND 31),
    next_payment_date DATE,
    end_date DATE,
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'cancelled')),
    
    -- Stats
    payments_completed INTEGER DEFAULT 0,
    total_collected DECIMAL(12,2) DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payment_schedules_tenant 
    ON payment_schedules(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payment_schedules_next_date 
    ON payment_schedules(next_payment_date) 
    WHERE status = 'active';

-- ============================================================================
-- Requirement EntityType (Buyer/Renter Requirements)
-- ============================================================================

-- Insert Requirement entity type metadata
INSERT INTO entity_types (app_id, code, name, icon, description, is_active, tenant_id)
SELECT 
    (SELECT id FROM apps WHERE code = 'real_estate' LIMIT 1),
    'requirement',
    'Requirement',
    '📋',
    'Buyer or renter property requirements for matching',
    TRUE,
    id
FROM tenants
ON CONFLICT DO NOTHING;

-- Requirement field definitions
INSERT INTO field_defs (entity_type_id, code, label, data_type, is_required, is_system, sort_order, tenant_id)
SELECT 
    et.id,
    fd.code,
    fd.label,
    fd.data_type,
    fd.is_required,
    fd.is_system,
    fd.sort_order,
    et.tenant_id
FROM entity_types et
CROSS JOIN (VALUES
    ('contact_id', 'Contact', 'uuid', TRUE, TRUE, 1),
    ('property_type', 'Property Type', 'select', FALSE, FALSE, 2),
    ('budget_min', 'Min Budget', 'currency', FALSE, FALSE, 3),
    ('budget_max', 'Max Budget', 'currency', FALSE, FALSE, 4),
    ('bedrooms_min', 'Min Bedrooms', 'number', FALSE, FALSE, 5),
    ('bathrooms_min', 'Min Bathrooms', 'number', FALSE, FALSE, 6),
    ('preferred_locations', 'Preferred Locations', 'text', FALSE, FALSE, 7),
    ('latitude', 'Latitude', 'number', FALSE, TRUE, 8),
    ('longitude', 'Longitude', 'number', FALSE, TRUE, 9),
    ('search_radius_km', 'Search Radius (km)', 'number', FALSE, FALSE, 10),
    ('amenities', 'Required Amenities', 'multiselect', FALSE, FALSE, 11),
    ('move_in_date', 'Desired Move-in Date', 'date', FALSE, FALSE, 12),
    ('notes', 'Notes', 'text', FALSE, FALSE, 13),
    ('status', 'Status', 'select', FALSE, FALSE, 14),
    ('matched_properties', 'Matched Properties', 'number', FALSE, TRUE, 15)
) AS fd(code, label, data_type, is_required, is_system, sort_order)
WHERE et.code = 'requirement'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Triggers
-- ============================================================================

CREATE OR REPLACE FUNCTION update_payment_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_payment_merchants_updated
    BEFORE UPDATE ON payment_merchants
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_timestamp();

CREATE TRIGGER tr_payments_updated
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_timestamp();

CREATE TRIGGER tr_payment_schedules_updated
    BEFORE UPDATE ON payment_schedules
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_timestamp();
