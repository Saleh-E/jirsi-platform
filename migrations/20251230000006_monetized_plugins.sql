-- ============================================================================
-- Monetized Plugin Economy
-- Enables paid apps and developer revenue share
-- ============================================================================

-- Add pricing and developer fields to apps
ALTER TABLE apps ADD COLUMN IF NOT EXISTS price_monthly DECIMAL(10,2) DEFAULT 0.00;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS price_yearly DECIMAL(10,2) DEFAULT 0.00;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS developer_id UUID;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS is_paid BOOLEAN DEFAULT FALSE;
ALTER TABLE apps ADD COLUMN IF NOT EXISTS revenue_share_percent DECIMAL(5,2) DEFAULT 80.00;  -- 80% to developer

-- Developer accounts (publishers)
CREATE TABLE IF NOT EXISTS developer_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- User link
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Business info
    business_name VARCHAR(255) NOT NULL,
    business_email VARCHAR(255) NOT NULL,
    business_type VARCHAR(50) DEFAULT 'individual',
    
    -- Stripe Connect for payouts
    stripe_account_id VARCHAR(100),
    payout_enabled BOOLEAN DEFAULT FALSE,
    
    -- Verification
    verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    
    -- Stats
    total_apps INTEGER DEFAULT 0,
    total_revenue DECIMAL(12,2) DEFAULT 0.00,
    total_paid_out DECIMAL(12,2) DEFAULT 0.00,
    
    -- Terms
    terms_accepted_at TIMESTAMPTZ,
    terms_version VARCHAR(50),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_developer_accounts_user 
    ON developer_accounts(user_id);

-- App subscriptions (tenant subscriptions to paid apps)
CREATE TABLE IF NOT EXISTS app_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    
    -- Subscription details
    plan VARCHAR(50) NOT NULL DEFAULT 'monthly'
        CHECK (plan IN ('monthly', 'yearly', 'lifetime')),
    status VARCHAR(50) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'cancelled', 'past_due', 'trialing', 'expired')),
    
    -- Pricing at time of subscription
    price_at_signup DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    
    -- Stripe subscription
    stripe_subscription_id VARCHAR(100),
    stripe_customer_id VARCHAR(100),
    
    -- Billing cycle
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    current_period_end TIMESTAMPTZ NOT NULL,
    trial_end TIMESTAMPTZ,
    
    -- Cancellation
    cancel_at_period_end BOOLEAN DEFAULT FALSE,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, app_id)
);

CREATE INDEX IF NOT EXISTS idx_app_subscriptions_tenant 
    ON app_subscriptions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_app_subscriptions_app 
    ON app_subscriptions(app_id);
CREATE INDEX IF NOT EXISTS idx_app_subscriptions_billing 
    ON app_subscriptions(current_period_end) 
    WHERE status = 'active';

-- Subscription invoices
CREATE TABLE IF NOT EXISTS app_subscription_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES app_subscriptions(id) ON DELETE CASCADE,
    
    -- Invoice details
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'failed', 'refunded')),
    
    -- Stripe
    stripe_invoice_id VARCHAR(100),
    stripe_payment_intent_id VARCHAR(100),
    
    -- Revenue split
    developer_amount DECIMAL(10,2) NOT NULL,
    platform_amount DECIMAL(10,2) NOT NULL,
    
    -- Payout status
    developer_payout_status VARCHAR(50) DEFAULT 'pending'
        CHECK (developer_payout_status IN ('pending', 'scheduled', 'paid', 'failed')),
    developer_payout_at TIMESTAMPTZ,
    
    -- Timing
    invoice_date DATE NOT NULL DEFAULT CURRENT_DATE,
    paid_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscription_invoices_subscription 
    ON app_subscription_invoices(subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscription_invoices_payout 
    ON app_subscription_invoices(developer_payout_status) 
    WHERE developer_payout_status = 'pending';

-- ============================================================================
-- Legal Compliance Fields
-- ============================================================================

-- Add legal consent fields to payment_merchants
ALTER TABLE payment_merchants 
    ADD COLUMN IF NOT EXISTS terms_accepted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS terms_version VARCHAR(50),
    ADD COLUMN IF NOT EXISTS privacy_accepted_at TIMESTAMPTZ;

-- Add voice recording consent to users
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS voice_recording_consent BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS voice_consent_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS marketing_consent BOOLEAN DEFAULT FALSE;

-- ============================================================================
-- Billing Job Support
-- ============================================================================

-- Track billing job runs
CREATE TABLE IF NOT EXISTS billing_job_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    job_type VARCHAR(50) NOT NULL,  -- 'subscription_billing', 'developer_payout'
    run_date DATE NOT NULL DEFAULT CURRENT_DATE,
    
    -- Stats
    processed_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    total_amount DECIMAL(12,2) DEFAULT 0.00,
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    -- Errors
    errors JSONB DEFAULT '[]',
    
    UNIQUE(job_type, run_date)
);

-- ============================================================================
-- Functions
-- ============================================================================

-- Function to process subscription billing
CREATE OR REPLACE FUNCTION process_subscription_billing()
RETURNS TABLE(processed INTEGER, billed INTEGER, failed INTEGER) AS $$
DECLARE
    sub RECORD;
    v_processed INTEGER := 0;
    v_billed INTEGER := 0;
    v_failed INTEGER := 0;
BEGIN
    -- Find subscriptions due for billing
    FOR sub IN 
        SELECT s.*, a.developer_id, a.revenue_share_percent
        FROM app_subscriptions s
        JOIN apps a ON s.app_id = a.id
        WHERE s.status = 'active'
        AND s.current_period_end <= NOW()
        AND NOT s.cancel_at_period_end
    LOOP
        v_processed := v_processed + 1;
        
        -- Calculate revenue split
        DECLARE
            developer_share DECIMAL := sub.price_at_signup * (sub.revenue_share_percent / 100);
            platform_share DECIMAL := sub.price_at_signup - developer_share;
        BEGIN
            -- Create invoice
            INSERT INTO app_subscription_invoices (
                subscription_id, amount, currency, 
                developer_amount, platform_amount, status
            ) VALUES (
                sub.id, sub.price_at_signup, sub.currency,
                developer_share, platform_share, 'pending'
            );
            
            -- Update subscription period
            UPDATE app_subscriptions SET
                current_period_start = NOW(),
                current_period_end = CASE 
                    WHEN sub.plan = 'monthly' THEN NOW() + INTERVAL '1 month'
                    WHEN sub.plan = 'yearly' THEN NOW() + INTERVAL '1 year'
                    ELSE current_period_end
                END,
                updated_at = NOW()
            WHERE id = sub.id;
            
            v_billed := v_billed + 1;
        EXCEPTION WHEN OTHERS THEN
            v_failed := v_failed + 1;
        END;
    END LOOP;
    
    RETURN QUERY SELECT v_processed, v_billed, v_failed;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Triggers
-- ============================================================================

CREATE TRIGGER tr_developer_accounts_updated
    BEFORE UPDATE ON developer_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_timestamp();

CREATE TRIGGER tr_app_subscriptions_updated
    BEFORE UPDATE ON app_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_timestamp();
