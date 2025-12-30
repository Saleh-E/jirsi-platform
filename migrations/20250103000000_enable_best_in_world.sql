-- Migration: Enable "Best in World" Features
-- Date: 2025-01-03
-- Purpose: Add Persona, Real Estate, and Marketplace capabilities

-- ============================================================================
-- 1. USERS: Add Persona & FinTech Fields
-- ============================================================================
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS verification_level INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS phone VARCHAR(50),
ADD COLUMN IF NOT EXISTS stripe_account_id VARCHAR(100);

-- Add index for phone lookups (WhatsApp matching)
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone) WHERE phone IS NOT NULL;

-- ============================================================================
-- 2. ENTITY TYPES: Add Real Estate Capability Flags
-- ============================================================================
ALTER TABLE entity_types
ADD COLUMN IF NOT EXISTS is_publishable BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS has_geo BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS has_gallery BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS is_contract BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS has_payments BOOLEAN DEFAULT false;

-- ============================================================================
-- 3. APPS: Add Marketplace Metadata
-- ============================================================================
-- Note: Check if table exists first since it might be named differently
DO $$ 
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'apps') THEN
        ALTER TABLE apps
        ADD COLUMN IF NOT EXISTS marketplace_id VARCHAR(100),
        ADD COLUMN IF NOT EXISTS version VARCHAR(20) DEFAULT '1.0.0',
        ADD COLUMN IF NOT EXISTS publisher VARCHAR(100) DEFAULT 'Local',
        ADD COLUMN IF NOT EXISTS auto_update BOOLEAN DEFAULT false;
    END IF;
END $$;

-- ============================================================================
-- 4. MARKETPLACE LISTINGS: Central Registry for Installable Apps
-- ============================================================================
CREATE TABLE IF NOT EXISTS marketplace_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id VARCHAR(50) NOT NULL UNIQUE,  -- e.g., 'com.jirsi.crm'
    name VARCHAR(100) NOT NULL,
    description TEXT,
    icon_url VARCHAR(255),
    
    -- Pricing
    price_monthly DECIMAL(10, 2) DEFAULT 0,
    is_free BOOLEAN DEFAULT true,
    
    -- Trust & Verification
    is_verified BOOLEAN DEFAULT false,
    publisher_name VARCHAR(100),
    publisher_url VARCHAR(255),
    
    -- Stats
    install_count INTEGER DEFAULT 0,
    rating DECIMAL(2, 1) DEFAULT 0,
    
    -- Metadata
    categories JSONB DEFAULT '[]',
    required_permissions JSONB DEFAULT '[]',
    supported_entity_types JSONB DEFAULT '[]',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_marketplace_verified ON marketplace_listings(is_verified) WHERE is_verified = true;
CREATE INDEX IF NOT EXISTS idx_marketplace_category ON marketplace_listings USING GIN(categories);

-- ============================================================================
-- 5. PROPERTY: Ensure geo columns exist for Map View
-- ============================================================================
-- Add latitude/longitude to entities table if using flexible schema
-- Or ensure the fields exist in field_defs for Property entity type

DO $$ 
BEGIN
    -- Check if we need to add geo fields to a properties-specific table
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'properties') THEN
        ALTER TABLE properties
        ADD COLUMN IF NOT EXISTS latitude DECIMAL(10, 8),
        ADD COLUMN IF NOT EXISTS longitude DECIMAL(11, 8);
        
        -- Note: For PostGIS spatial indexing, install PostGIS extension first:
        -- CREATE EXTENSION IF NOT EXISTS postgis;
        -- Then add: ADD COLUMN IF NOT EXISTS geo_point GEOGRAPHY(POINT, 4326);
        -- CREATE INDEX IF NOT EXISTS idx_properties_geo ON properties USING GIST(geo_point);
    END IF;
END $$;

-- ============================================================================
-- ROLLBACK (if needed)
-- ============================================================================
-- ALTER TABLE users DROP COLUMN IF EXISTS verification_level;
-- ALTER TABLE users DROP COLUMN IF EXISTS phone;
-- ALTER TABLE users DROP COLUMN IF EXISTS stripe_account_id;
-- ALTER TABLE entity_types DROP COLUMN IF EXISTS is_publishable;
-- ALTER TABLE entity_types DROP COLUMN IF EXISTS has_geo;
-- ALTER TABLE entity_types DROP COLUMN IF EXISTS has_gallery;
-- ALTER TABLE entity_types DROP COLUMN IF EXISTS is_contract;
-- ALTER TABLE entity_types DROP COLUMN IF EXISTS has_payments;
-- DROP TABLE IF EXISTS marketplace_listings;
