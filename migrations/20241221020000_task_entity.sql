-- Create Task entity type and table
-- Tasks are linked to any entity and used for follow-ups, reminders, etc.

-- ============================================================================
-- TASK ENTITY TYPE
-- ============================================================================

INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon)
VALUES (
    'e0000000-0000-0000-0000-000000000005',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'crm',
    'task',
    'Task',
    'Tasks',
    'check-square'
)
ON CONFLICT (tenant_id, name) DO NOTHING;

-- ============================================================================
-- TASK FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder, options)
VALUES
-- Basic Info
('f5000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'title', 'Title', '{"type": "Text"}', true, true, true, false, 1, 'Basic', 'Task title', NULL),

('f5000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'description', 'Description', '{"type": "LongText"}', false, false, false, false, 2, 'Basic', 'Task description', NULL),

-- Status & Priority
('f5000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'status', 'Status', '{"type": "Select"}', true, true, true, false, 3, 'Status', NULL,
 '[{"value": "pending", "label": "Pending"}, {"value": "in_progress", "label": "In Progress"}, {"value": "completed", "label": "Completed"}, {"value": "cancelled", "label": "Cancelled"}]'),

('f5000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'priority', 'Priority', '{"type": "Select"}', true, true, true, false, 4, 'Status', NULL,
 '[{"value": "low", "label": "Low"}, {"value": "medium", "label": "Medium"}, {"value": "high", "label": "High"}, {"value": "urgent", "label": "Urgent"}]'),

-- Dates
('f5000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'due_date', 'Due Date', '{"type": "Date"}', false, true, true, false, 5, 'Dates', NULL, NULL),

('f5000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'completed_at', 'Completed At', '{"type": "DateTime"}', false, false, false, true, 6, 'Dates', NULL, NULL),

-- Associations
('f5000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'assigned_to', 'Assigned To', '{"type": "Lookup"}', false, true, false, false, 7, 'Assignment', NULL, NULL),

('f5000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'linked_entity_type', 'Linked Entity Type', '{"type": "Text"}', false, false, false, false, 8, 'Links', NULL, NULL),

('f5000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000005', 
 'linked_entity_id', 'Linked Entity ID', '{"type": "Text"}', false, false, false, false, 9, 'Links', NULL, NULL)

ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- TASKS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(30) DEFAULT 'pending',
    priority VARCHAR(20) DEFAULT 'medium',
    due_date DATE,
    completed_at TIMESTAMPTZ,
    assigned_to UUID,
    linked_entity_type VARCHAR(50),
    linked_entity_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tasks_tenant ON tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned ON tasks(assigned_to);
CREATE INDEX IF NOT EXISTS idx_tasks_linked ON tasks(linked_entity_type, linked_entity_id);
CREATE INDEX IF NOT EXISTS idx_tasks_deleted ON tasks(deleted_at) WHERE deleted_at IS NULL;

-- ============================================================================
-- DEFAULT VIEW FOR TASKS
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_shared, settings)
VALUES (
    'v0000000-0000-0000-0000-000000000050',
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'e0000000-0000-0000-0000-000000000005',
    'all_tasks',
    'All Tasks',
    'table',
    true,
    true,
    '{}'::jsonb
)
ON CONFLICT DO NOTHING;
