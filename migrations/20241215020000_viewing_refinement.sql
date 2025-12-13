-- Phase 3 Task P3-02: Viewing Entity Refinement
-- Updates Viewing entity with scheduled_start/end and calendar integration

-- ============================================================================
-- UPDATE VIEWING TABLE STRUCTURE
-- ============================================================================

-- Add scheduled_start and scheduled_end columns
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS scheduled_start TIMESTAMPTZ;
ALTER TABLE viewings ADD COLUMN IF NOT EXISTS scheduled_end TIMESTAMPTZ;

-- Migrate data from scheduled_at to scheduled_start if it exists
UPDATE viewings 
SET scheduled_start = scheduled_at,
    scheduled_end = scheduled_at + INTERVAL '1 hour'
WHERE scheduled_start IS NULL AND scheduled_at IS NOT NULL;

-- ============================================================================
-- UPDATE/INSERT FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, is_readonly, sort_order, "group", placeholder, options)
VALUES
('f8000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'property_id', 'Property', 'lookup', true, true, true, false, 1, 'Basic', NULL, NULL),
('f8000002-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'contact_id', 'Contact', 'lookup', true, true, true, false, 2, 'Basic', NULL, NULL),
('f8000002-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'agent_id', 'Agent', 'lookup', false, true, false, false, 3, 'Basic', NULL, NULL),
('f8000002-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'scheduled_start', 'Start Time', 'datetime', true, true, true, false, 4, 'Schedule', NULL, NULL),
('f8000002-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'scheduled_end', 'End Time', 'datetime', true, true, false, false, 5, 'Schedule', NULL, NULL),
('f8000002-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'status', 'Status', 'select', true, true, true, false, 6, 'Status', NULL,
 '[{"value": "scheduled", "label": "Scheduled"}, {"value": "completed", "label": "Completed"}, {"value": "noshow", "label": "No Show"}, {"value": "cancelled", "label": "Cancelled"}]'::jsonb),
('f8000002-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011', 
 'notes', 'Notes', 'textarea', false, false, false, false, 7, 'Details', NULL, NULL)
ON CONFLICT (entity_type_id, name) DO UPDATE SET
    label = EXCLUDED.label,
    field_type = EXCLUDED.field_type,
    is_required = EXCLUDED.is_required,
    show_in_list = EXCLUDED.show_in_list,
    sort_order = EXCLUDED.sort_order,
    options = EXCLUDED.options;

-- ============================================================================
-- UPDATE DEFAULT VIEWS FOR VIEWING
-- ============================================================================

-- Update table view columns
UPDATE view_defs 
SET columns = '["property_id", "contact_id", "agent_id", "scheduled_start", "scheduled_end", "status"]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000011' AND view_type = 'table';

-- Update Calendar view with start/end fields
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'viewings_calendar', 'Calendar', 'calendar', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"date_field": "scheduled_start", "end_date_field": "scheduled_end", "title_field": "property_id", "color_field": "status", "color_map": {"scheduled": "#3b82f6", "completed": "#22c55e", "noshow": "#f59e0b", "cancelled": "#ef4444"}}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- Kanban view by status
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000002-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'viewings_pipeline', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{"group_by_field": "status", "card_title_field": "property_id", "card_subtitle_field": "contact_id", "card_fields": ["scheduled_start", "agent_id"], "column_order": ["scheduled", "completed", "noshow", "cancelled"]}'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    settings = EXCLUDED.settings;

-- ============================================================================
-- CREATE INDEXES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_viewings_scheduled_start ON viewings(scheduled_start);
CREATE INDEX IF NOT EXISTS idx_viewings_scheduled_end ON viewings(scheduled_end);
CREATE INDEX IF NOT EXISTS idx_viewings_agent_time ON viewings(agent_id, scheduled_start, scheduled_end);
