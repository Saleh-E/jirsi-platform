-- Phase 3 Task P3-08: Map View for Properties
-- Configure Map view using property lat/long coordinates

-- ============================================================================
-- PROPERTY MAP VIEW
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000008-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010',
 'properties_map', 'Map', 'map', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{
   "lat_field": "latitude",
   "lng_field": "longitude",
   "popup_title_field": "title",
   "popup_fields": ["price", "bedrooms", "bathrooms", "property_type", "status"],
   "marker_color_field": "status",
   "marker_colors": {
     "draft": "#6b7280",
     "active": "#22c55e",
     "reserved": "#f59e0b",
     "under_offer": "#8b5cf6",
     "sold": "#10b981",
     "rented": "#06b6d4",
     "withdrawn": "#ef4444"
   },
   "default_center": [25.2048, 55.2708],
   "default_zoom": 11,
   "enable_clustering": true,
   "cluster_radius": 50
 }'::jsonb)
ON CONFLICT (id) DO UPDATE SET settings = EXCLUDED.settings;

-- ============================================================================
-- ENSURE PROPERTIES HAVE LAT/LONG FIELDS
-- ============================================================================

-- Add latitude and longitude columns if not exists
ALTER TABLE properties ADD COLUMN IF NOT EXISTS latitude DECIMAL(10, 8);
ALTER TABLE properties ADD COLUMN IF NOT EXISTS longitude DECIMAL(11, 8);

-- Add field definitions for lat/long
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group")
VALUES
('f8000008-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 
 'latitude', 'Latitude', 'number', false, false, false, false, 40, 'Location'),
('f8000008-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 
 'longitude', 'Longitude', 'number', false, false, false, false, 41, 'Location')
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- UPDATE SAMPLE PROPERTIES WITH COORDINATES (Dubai locations)
-- ============================================================================

UPDATE properties SET 
    latitude = 25.2048 + (random() * 0.1 - 0.05),
    longitude = 55.2708 + (random() * 0.1 - 0.05)
WHERE latitude IS NULL AND tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';

-- ============================================================================
-- CREATE INDEX FOR GEOGRAPHIC QUERIES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_properties_coordinates ON properties(latitude, longitude);
