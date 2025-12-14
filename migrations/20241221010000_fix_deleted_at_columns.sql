-- Fix missing columns on offers, viewings, contracts, and listings tables
-- Drop and recreate to ensure correct schema

-- ============================================================================
-- FIX OFFERS TABLE
-- ============================================================================
ALTER TABLE offers ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS offer_amount DECIMAL(15, 2);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS currency VARCHAR(3) DEFAULT 'USD';
ALTER TABLE offers ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS conditions TEXT;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS counter_amount DECIMAL(15, 2);
ALTER TABLE offers ADD COLUMN IF NOT EXISTS accepted_at TIMESTAMPTZ;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS rejected_reason TEXT;
ALTER TABLE offers ADD COLUMN IF NOT EXISTS deal_id UUID;

-- ============================================================================
-- FIX VIEWINGS TABLE
-- ============================================================================
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS duration_minutes INTEGER DEFAULT 30;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS feedback TEXT;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS rating INTEGER;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS follow_up_notes TEXT;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS scheduled_start TIMESTAMPTZ;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS scheduled_end TIMESTAMPTZ;

-- ============================================================================
-- FIX CONTRACTS TABLE (if exists)
-- ============================================================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'contracts') THEN
        ALTER TABLE contracts ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
    END IF;
END $$;

-- ============================================================================
-- FIX LISTINGS TABLE (if exists)
-- ============================================================================
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'listings') THEN
        ALTER TABLE listings ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
    END IF;
END $$;

-- Create indexes for soft delete queries
CREATE INDEX IF NOT EXISTS idx_offers_deleted ON offers(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_viewings_deleted ON viewings(deleted_at) WHERE deleted_at IS NULL;
