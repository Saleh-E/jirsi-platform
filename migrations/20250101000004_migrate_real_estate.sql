-- Migrate Properties and Viewings to entity_records

DO $$
DECLARE
    real_estate_app_id UUID;
    property_type_id UUID;
    viewing_type_id UUID;
BEGIN
    -- Ensure entity types exist
    SELECT id INTO property_type_id FROM entity_types WHERE name = 'property' LIMIT 1;
    SELECT id INTO viewing_type_id FROM entity_types WHERE name = 'viewing' LIMIT 1;

    -- Migrate Properties
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'properties') THEN
        EXECUTE '
            INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at, deleted_at)
            SELECT 
                id,
                tenant_id,
                $1,
                jsonb_build_object(
                    ''reference'', reference,
                    ''title'', title,
                    ''description'', description,
                    ''property_type'', property_type,
                    ''usage'', usage,
                    ''status'', status,
                    ''address'', address,
                    ''region'', area,
                    ''bedrooms'', bedrooms,
                    ''bathrooms'', bathrooms,
                    ''size_sqm'', size_sqm,
                    ''price'', price,
                    ''rent_amount'', rent_amount,
                    ''currency'', currency,
                    ''amenities'', amenities,
                    ''images'', photos,
                    ''owner_id'', owner_id
                ),
                created_at,
                updated_at,
                deleted_at
            FROM properties
        ' USING property_type_id;
    END IF;

    -- Migrate Viewings
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'viewings') THEN
        EXECUTE '
            INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at, deleted_at)
            SELECT 
                id,
                tenant_id,
                $1,
                jsonb_build_object(
                    ''property_id'', property_id,
                    ''contact_id'', contact_id,
                    ''agent_id'', agent_id,
                    ''scheduled_at'', scheduled_at,
                    ''duration_minutes'', duration_minutes,
                    ''status'', status,
                    ''feedback'', feedback,
                    ''rating'', rating
                ),
                created_at,
                updated_at,
                deleted_at
            FROM viewings
        ' USING viewing_type_id;
    END IF;

END $$;
