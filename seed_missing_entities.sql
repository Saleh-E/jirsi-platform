-- Add missing entity types for tenant c65f0c95-f439-4554-a9fb-4ee7b1785ebf

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags) VALUES
(gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', 'realestate', 'property', 'Property', 'Properties', 'home', '{"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true, "has_map": true}'),
(gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', 'realestate', 'listing', 'Listing', 'Listings', 'clipboard-list', '{"has_activities": true, "is_searchable": true, "show_in_nav": true}'),
(gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', 'realestate', 'viewing', 'Viewing', 'Viewings', 'eye', '{"has_activities": true, "show_in_nav": true}'),
(gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', 'realestate', 'offer', 'Offer', 'Offers', 'dollar-sign', '{"has_activities": true, "show_in_nav": true}'),
(gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', 'crm', 'task', 'Task', 'Tasks', 'check-square', '{"show_in_nav": true}')
ON CONFLICT DO NOTHING;

-- Add field definitions for property
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'title', 'Title', 'text', true, true, 1 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'address', 'Address', 'text', false, true, 2 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'price', 'Price', 'money', false, true, 3 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'property_type', 'Property Type', 'select', false, true, 4 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'status', 'Status', 'status', false, true, 5 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'bedrooms', 'Bedrooms', 'number', false, true, 6 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'bathrooms', 'Bathrooms', 'number', false, true, 7 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'city', 'City', 'text', false, true, 8 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'property';

-- Add field definitions for listing
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'title', 'Title', 'text', true, true, 1 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'listing';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'status', 'Status', 'status', false, true, 2 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'listing';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'price', 'Price', 'money', false, true, 3 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'listing';

-- Add field definitions for viewing
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'scheduled_date', 'Scheduled Date', 'datetime', true, true, 1 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'viewing';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'status', 'Status', 'status', false, true, 2 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'viewing';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'notes', 'Notes', 'textarea', false, false, 3 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'viewing';

-- Add field definitions for offer
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'amount', 'Offer Amount', 'money', true, true, 1 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'offer';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'status', 'Status', 'status', false, true, 2 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'offer';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'offer_date', 'Offer Date', 'date', false, true, 3 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'offer';

-- Add field definitions for task
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'title', 'Title', 'text', true, true, 1 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'task';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'status', 'Status', 'status', false, true, 2 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'task';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'due_date', 'Due Date', 'date', false, true, 3 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'task';

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
SELECT gen_random_uuid(), 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf', id, 'description', 'Description', 'textarea', false, false, 4 FROM entity_types WHERE tenant_id = 'c65f0c95-f439-4554-a9fb-4ee7b1785ebf' AND name = 'task';
