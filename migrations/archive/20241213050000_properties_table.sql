-- Phase 1B Task P1B-05: Properties Table Schema
-- Creates the properties table with all columns matching FieldDefs

-- Drop existing table if it exists (for clean slate)
DROP TABLE IF EXISTS properties CASCADE;

CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Basic Info
    reference VARCHAR(50),
    title VARCHAR(255) NOT NULL,
    property_type VARCHAR(50),
    usage VARCHAR(20),
    status VARCHAR(30) DEFAULT 'draft',
    
    -- Location
    country VARCHAR(100),
    city VARCHAR(100),
    area VARCHAR(200),
    address TEXT,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    
    -- Specifications
    bedrooms INTEGER,
    bathrooms INTEGER,
    size_sqm DECIMAL(12, 2),
    floor INTEGER,
    total_floors INTEGER,
    year_built INTEGER,
    
    -- Financial
    price DECIMAL(15, 2),
    rent_amount DECIMAL(15, 2),
    currency VARCHAR(3) DEFAULT 'USD',
    service_charge DECIMAL(15, 2),
    commission_percent DECIMAL(5, 2),
    
    -- Relations
    owner_id UUID,
    agent_id UUID,
    developer_id UUID,
    
    -- Details
    description TEXT,
    amenities JSONB DEFAULT '[]',
    
    -- Media
    photos JSONB DEFAULT '[]',
    documents JSONB DEFAULT '[]',
    
    -- Dates
    listed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes for common queries
CREATE INDEX idx_properties_tenant ON properties(tenant_id);
CREATE INDEX idx_properties_reference ON properties(tenant_id, reference);
CREATE INDEX idx_properties_status ON properties(tenant_id, status);
CREATE INDEX idx_properties_city ON properties(tenant_id, city);
CREATE INDEX idx_properties_usage ON properties(tenant_id, usage);
CREATE INDEX idx_properties_property_type ON properties(tenant_id, property_type);
CREATE INDEX idx_properties_price ON properties(price) WHERE price IS NOT NULL;
CREATE INDEX idx_properties_rent ON properties(rent_amount) WHERE rent_amount IS NOT NULL;
CREATE INDEX idx_properties_geo ON properties(latitude, longitude) WHERE latitude IS NOT NULL AND longitude IS NOT NULL;
CREATE INDEX idx_properties_deleted ON properties(deleted_at) WHERE deleted_at IS NULL;
