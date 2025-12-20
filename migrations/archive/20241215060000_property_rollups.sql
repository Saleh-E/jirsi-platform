-- Phase 3 Task P3-06: Property Rollups
-- Add computed rollup fields to Property for viewings/offers counts

-- ============================================================================
-- ADD ROLLUP COLUMNS TO PROPERTIES
-- ============================================================================

ALTER TABLE properties ADD COLUMN IF NOT EXISTS viewing_count INTEGER DEFAULT 0;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS offer_count INTEGER DEFAULT 0;
ALTER TABLE properties ADD COLUMN IF NOT EXISTS active_listing_count INTEGER DEFAULT 0;

-- ============================================================================
-- CREATE ROLLUP TRIGGERS
-- ============================================================================

-- Function to update viewing count
CREATE OR REPLACE FUNCTION update_property_viewing_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE properties 
        SET viewing_count = viewing_count + 1,
            updated_at = NOW()
        WHERE id = NEW.property_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE properties 
        SET viewing_count = GREATEST(0, viewing_count - 1),
            updated_at = NOW()
        WHERE id = OLD.property_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update offer count
CREATE OR REPLACE FUNCTION update_property_offer_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE properties 
        SET offer_count = offer_count + 1,
            updated_at = NOW()
        WHERE id = NEW.property_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE properties 
        SET offer_count = GREATEST(0, offer_count - 1),
            updated_at = NOW()
        WHERE id = OLD.property_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update active listing count
CREATE OR REPLACE FUNCTION update_property_listing_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.status = 'live' THEN
        UPDATE properties 
        SET active_listing_count = active_listing_count + 1,
            updated_at = NOW()
        WHERE id = NEW.property_id;
    ELSIF TG_OP = 'DELETE' AND OLD.status = 'live' THEN
        UPDATE properties 
        SET active_listing_count = GREATEST(0, active_listing_count - 1),
            updated_at = NOW()
        WHERE id = OLD.property_id;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Handle status change
        IF OLD.status = 'live' AND NEW.status != 'live' THEN
            UPDATE properties 
            SET active_listing_count = GREATEST(0, active_listing_count - 1),
                updated_at = NOW()
            WHERE id = NEW.property_id;
        ELSIF OLD.status != 'live' AND NEW.status = 'live' THEN
            UPDATE properties 
            SET active_listing_count = active_listing_count + 1,
                updated_at = NOW()
            WHERE id = NEW.property_id;
        END IF;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
DROP TRIGGER IF EXISTS trg_viewing_count ON viewings;
CREATE TRIGGER trg_viewing_count
    AFTER INSERT OR DELETE ON viewings
    FOR EACH ROW EXECUTE FUNCTION update_property_viewing_count();

DROP TRIGGER IF EXISTS trg_offer_count ON offers;
CREATE TRIGGER trg_offer_count
    AFTER INSERT OR DELETE ON offers
    FOR EACH ROW EXECUTE FUNCTION update_property_offer_count();

DROP TRIGGER IF EXISTS trg_listing_count ON listings;
CREATE TRIGGER trg_listing_count
    AFTER INSERT OR DELETE OR UPDATE OF status ON listings
    FOR EACH ROW EXECUTE FUNCTION update_property_listing_count();

-- ============================================================================
-- ADD ROLLUP FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group")
VALUES
('f8000005-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 
 'viewing_count', 'Viewings', 'integer', false, true, true, true, 50, 'Stats'),
('f8000005-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 
 'offer_count', 'Offers', 'integer', false, true, true, true, 51, 'Stats'),
('f8000005-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 
 'active_listing_count', 'Active Listings', 'integer', false, true, false, true, 52, 'Stats')
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- INITIALIZE ROLLUP COUNTS FROM EXISTING DATA
-- ============================================================================

UPDATE properties p SET 
    viewing_count = (SELECT COUNT(*) FROM viewings v WHERE v.property_id = p.id),
    offer_count = (SELECT COUNT(*) FROM offers o WHERE o.property_id = p.id),
    active_listing_count = (SELECT COUNT(*) FROM listings l WHERE l.property_id = p.id AND l.status = 'live');
