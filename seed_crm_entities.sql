-- Add missing CRM entity types for tenant b128c8da-6e56-485d-b2fe-e45fb7492b2e

-- Contact
INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags)
VALUES ('e0000000-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'crm', 'contact', 'Contact', 'Contacts', 'user', '{"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}')
ON CONFLICT DO NOTHING;

-- Company
INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags)
VALUES ('e0000000-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'crm', 'company', 'Company', 'Companies', 'building', '{"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}')
ON CONFLICT DO NOTHING;

-- Deal
INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags)
VALUES ('e0000000-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'crm', 'deal', 'Deal', 'Deals', 'dollar-sign', '{"has_pipeline": true, "has_activities": true, "is_searchable": true, "show_in_nav": true}')
ON CONFLICT DO NOTHING;

-- Task
INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags)
VALUES ('e0000000-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'crm', 'task', 'Task', 'Tasks', 'check-square', '{"show_in_nav": true}')
ON CONFLICT DO NOTHING;

-- Contact fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
VALUES
('f0000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001', 'first_name', 'First Name', 'text', true, true, 1),
('f0000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001', 'last_name', 'Last Name', 'text', true, true, 2),
('f0000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001', 'email', 'Email', 'email', false, true, 3),
('f0000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001', 'phone', 'Phone', 'phone', false, true, 4),
('f0000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000001', 'lifecycle_stage', 'Lifecycle Stage', 'select', false, true, 5)
ON CONFLICT DO NOTHING;

-- Set lifecycle_stage options
UPDATE field_defs SET options = '[
    {"value": "subscriber", "label": "Subscriber"},
    {"value": "lead", "label": "Lead"},
    {"value": "mql", "label": "Marketing Qualified Lead"},
    {"value": "sql", "label": "Sales Qualified Lead"},
    {"value": "opportunity", "label": "Opportunity"},
    {"value": "customer", "label": "Customer"},
    {"value": "evangelist", "label": "Evangelist"}
]'::jsonb WHERE id = 'f0000001-0000-0000-0000-000000000005';

-- Company fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
VALUES
('f0000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000002', 'name', 'Company Name', 'text', true, true, 1),
('f0000002-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000002', 'domain', 'Domain', 'url', false, true, 2),
('f0000002-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000002', 'industry', 'Industry', 'select', false, true, 3),
('f0000002-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000002', 'phone', 'Phone', 'phone', false, true, 4)
ON CONFLICT DO NOTHING;

-- Set industry options
UPDATE field_defs SET options = '[
    {"value": "technology", "label": "Technology"},
    {"value": "finance", "label": "Finance"},
    {"value": "healthcare", "label": "Healthcare"},
    {"value": "retail", "label": "Retail"},
    {"value": "manufacturing", "label": "Manufacturing"},
    {"value": "real_estate", "label": "Real Estate"},
    {"value": "education", "label": "Education"},
    {"value": "other", "label": "Other"}
]'::jsonb WHERE id = 'f0000002-0000-0000-0000-000000000003';

-- Deal fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
VALUES
('f0000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000003', 'name', 'Deal Name', 'text', true, true, 1),
('f0000003-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000003', 'amount', 'Amount', 'currency', false, true, 2),
('f0000003-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000003', 'stage', 'Stage', 'select', true, true, 3),
('f0000003-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000003', 'expected_close_date', 'Expected Close', 'date', false, true, 4)
ON CONFLICT DO NOTHING;

-- Set deal stage options
UPDATE field_defs SET options = '[
    {"value": "lead", "label": "Lead"},
    {"value": "qualified", "label": "Qualified"},
    {"value": "proposal", "label": "Proposal"},
    {"value": "negotiation", "label": "Negotiation"},
    {"value": "closed_won", "label": "Closed Won"},
    {"value": "closed_lost", "label": "Closed Lost"}
]'::jsonb WHERE id = 'f0000003-0000-0000-0000-000000000003';

-- Task fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order)
VALUES
('f0000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004', 'title', 'Title', 'text', true, true, 1),
('f0000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004', 'status', 'Status', 'select', false, true, 2),
('f0000004-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004', 'due_date', 'Due Date', 'date', false, true, 3),
('f0000004-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004', 'priority', 'Priority', 'select', false, true, 4),
('f0000004-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000004', 'description', 'Description', 'textarea', false, false, 5)
ON CONFLICT DO NOTHING;

-- Set task status options
UPDATE field_defs SET options = '[
    {"value": "not_started", "label": "Not Started"},
    {"value": "in_progress", "label": "In Progress"},
    {"value": "waiting", "label": "Waiting"},
    {"value": "completed", "label": "Completed"},
    {"value": "deferred", "label": "Deferred"}
]'::jsonb WHERE id = 'f0000004-0000-0000-0000-000000000002';

-- Set task priority options
UPDATE field_defs SET options = '[
    {"value": "low", "label": "Low"},
    {"value": "normal", "label": "Normal"},
    {"value": "high", "label": "High"},
    {"value": "urgent", "label": "Urgent"}
]'::jsonb WHERE id = 'f0000004-0000-0000-0000-000000000004';

-- Sample contacts
INSERT INTO contacts (id, tenant_id, first_name, last_name, email, phone, lifecycle_stage)
VALUES
(gen_random_uuid(), 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'Alice', 'Williams', 'alice.williams@example.com', '+1-555-0104', 'lead'),
(gen_random_uuid(), 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'Bob', 'Johnson', 'bob.johnson@example.com', '+1-555-0102', 'lead'),
(gen_random_uuid(), 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'Charlie', 'Brown', 'charlie.brown@example.com', '+1-555-0103', 'lead'),
(gen_random_uuid(), 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'Jane', 'Doe', 'jane.doe@example.com', '+1-555-0105', 'lead'),
(gen_random_uuid(), 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'John', 'Smith', 'john.smith@example.com', '+1-555-0101', 'lead')
ON CONFLICT DO NOTHING;
