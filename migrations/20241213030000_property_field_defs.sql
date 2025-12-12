-- Phase 1B Task P1B-03: Property FieldDefs
-- Creates 30 field definitions for Property entity
-- Groups: Basic, Location, Specifications, Financial, Relations, Details, Media, Dates

-- BASIC INFO (5 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'reference', 'Reference', 'text', true, true, true, false, 1, 'Basic', 'PROP-001'),
('f1000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'title', 'Title', 'text', true, true, true, false, 2, 'Basic', 'Beach Villa'),
('f1000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'property_type', 'Property Type', 'select', true, true, true, false, 3, 'Basic', NULL),
('f1000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'usage', 'Usage', 'select', true, true, true, false, 4, 'Basic', NULL),
('f1000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'status', 'Status', 'select', true, true, true, false, 5, 'Basic', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- LOCATION (6 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'country', 'Country', 'text', false, false, false, false, 6, 'Location', 'UAE'),
('f1000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'city', 'City', 'text', true, true, true, false, 7, 'Location', 'Dubai'),
('f1000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'area', 'Area', 'text', false, true, false, false, 8, 'Location', 'Downtown'),
('f1000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'address', 'Address', 'textarea', false, false, false, false, 9, 'Location', NULL),
('f1000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'latitude', 'Latitude', 'number', false, false, false, false, 10, 'Location', NULL),
('f1000001-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'longitude', 'Longitude', 'number', false, false, false, false, 11, 'Location', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- SPECIFICATIONS (6 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'bedrooms', 'Bedrooms', 'integer', false, true, true, false, 12, 'Specifications', NULL),
('f1000001-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'bathrooms', 'Bathrooms', 'integer', false, true, false, false, 13, 'Specifications', NULL),
('f1000001-0000-0000-0000-000000000014', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'size_sqm', 'Size (sqm)', 'number', false, true, false, false, 14, 'Specifications', NULL),
('f1000001-0000-0000-0000-000000000015', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'floor', 'Floor', 'integer', false, false, false, false, 15, 'Specifications', NULL),
('f1000001-0000-0000-0000-000000000016', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'total_floors', 'Total Floors', 'integer', false, false, false, false, 16, 'Specifications', NULL),
('f1000001-0000-0000-0000-000000000017', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'year_built', 'Year Built', 'integer', false, false, false, false, 17, 'Specifications', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- FINANCIAL (5 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000018', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'price', 'Price', 'currency', false, true, true, false, 18, 'Financial', NULL),
('f1000001-0000-0000-0000-000000000019', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'rent_amount', 'Rent Amount', 'currency', false, true, false, false, 19, 'Financial', NULL),
('f1000001-0000-0000-0000-000000000020', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'currency', 'Currency', 'select', false, false, false, false, 20, 'Financial', NULL),
('f1000001-0000-0000-0000-000000000021', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'service_charge', 'Service Charge', 'currency', false, false, false, false, 21, 'Financial', NULL),
('f1000001-0000-0000-0000-000000000022', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'commission_percent', 'Commission %', 'number', false, false, false, false, 22, 'Financial', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- RELATIONS (3 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000023', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'owner_id', 'Owner', 'lookup', false, false, false, false, 23, 'Relations', NULL),
('f1000001-0000-0000-0000-000000000024', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'agent_id', 'Agent', 'lookup', false, false, false, false, 24, 'Relations', NULL),
('f1000001-0000-0000-0000-000000000025', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'developer_id', 'Developer', 'lookup', false, false, false, false, 25, 'Relations', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- DETAILS (2 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000026', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'description', 'Description', 'longtext', false, false, false, false, 26, 'Details', NULL),
('f1000001-0000-0000-0000-000000000027', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'amenities', 'Amenities', 'multiselect', false, false, false, false, 27, 'Details', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- MEDIA (2 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000028', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'photos', 'Photos', 'file_array', false, false, false, false, 28, 'Media', NULL),
('f1000001-0000-0000-0000-000000000029', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'documents', 'Documents', 'file_array', false, false, false, false, 29, 'Media', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- DATES (2 fields)
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder)
VALUES
('f1000001-0000-0000-0000-000000000030', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'listed_at', 'Listed At', 'datetime', false, false, false, false, 30, 'Dates', NULL),
('f1000001-0000-0000-0000-000000000031', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010', 'expires_at', 'Expires At', 'datetime', false, false, false, false, 31, 'Dates', NULL)
ON CONFLICT (entity_type_id, name) DO NOTHING;
