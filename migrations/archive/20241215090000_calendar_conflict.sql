-- Phase 3 Task P3-09: Calendar View with Conflict Check
-- Aggregates viewings and contracts with conflict detection

-- ============================================================================
-- AGGREGATE CALENDAR VIEW
-- Combines events from multiple entities
-- ============================================================================

CREATE OR REPLACE VIEW v_calendar_events AS
-- Viewings
SELECT 
    v.id,
    v.tenant_id,
    'viewing' as event_type,
    v.property_id as related_entity_id,
    'property' as related_entity_type,
    COALESCE(p.title, 'Property Viewing') as title,
    v.scheduled_start as start_time,
    v.scheduled_end as end_time,
    v.agent_id,
    v.status,
    CASE 
        WHEN v.status = 'scheduled' THEN '#3b82f6'
        WHEN v.status = 'completed' THEN '#22c55e'
        WHEN v.status = 'noshow' THEN '#f59e0b'
        WHEN v.status = 'cancelled' THEN '#ef4444'
        ELSE '#6b7280'
    END as color
FROM viewings v
LEFT JOIN properties p ON v.property_id = p.id

UNION ALL

-- Contracts (as date ranges)
SELECT 
    c.id,
    c.tenant_id,
    'contract' as event_type,
    c.property_id as related_entity_id,
    'property' as related_entity_type,
    COALESCE(p.title, 'Contract') || ' - ' || c.contract_type as title,
    c.start_date::timestamptz as start_time,
    COALESCE(c.end_date, c.start_date)::timestamptz as end_time,
    NULL::uuid as agent_id,
    c.status,
    CASE 
        WHEN c.status = 'draft' THEN '#6b7280'
        WHEN c.status = 'active' THEN '#22c55e'
        WHEN c.status = 'completed' THEN '#3b82f6'
        WHEN c.status = 'terminated' THEN '#ef4444'
        ELSE '#6b7280'
    END as color
FROM contracts c
LEFT JOIN properties p ON c.property_id = p.id;

-- ============================================================================
-- CONFLICT DETECTION FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION check_viewing_conflict(
    p_tenant_id UUID,
    p_agent_id UUID,
    p_start_time TIMESTAMPTZ,
    p_end_time TIMESTAMPTZ,
    p_exclude_id UUID DEFAULT NULL
) RETURNS TABLE (
    conflicting_viewing_id UUID,
    conflicting_start TIMESTAMPTZ,
    conflicting_end TIMESTAMPTZ,
    property_title VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        v.id,
        v.scheduled_start,
        v.scheduled_end,
        p.title
    FROM viewings v
    LEFT JOIN properties p ON v.property_id = p.id
    WHERE v.tenant_id = p_tenant_id
      AND v.agent_id = p_agent_id
      AND v.status IN ('scheduled')
      AND v.id != COALESCE(p_exclude_id, '00000000-0000-0000-0000-000000000000')
      AND (
          (v.scheduled_start < p_end_time AND v.scheduled_end > p_start_time)
      );
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- CALENDAR AGGREGATE ENDPOINT VIEW
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Unified calendar view (special entity_type_id for aggregate)
('b8000009-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000011',
 'viewings_unified_calendar', 'Unified Calendar', 'calendar', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{
   "sources": ["viewing", "contract"],
   "date_field": "start_time",
   "end_date_field": "end_time",
   "title_field": "title",
   "color_field": "color",
   "conflict_check": true,
   "conflict_field": "agent_id"
 }'::jsonb)
ON CONFLICT (id) DO UPDATE SET settings = EXCLUDED.settings;
